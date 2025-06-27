use std::io::{self, BufRead, BufReader, Cursor};

use git_packetline::{PacketLineRef, StreamingPeekableIter};
use log::Log;


pub fn parse_ref_advertisement(bytes: &[u8]) -> std::io::Result<RefAdvertisement> {
    let reader =    BufReader::new(Cursor::new(bytes));
    let mut packets = StreamingPeekableIter::new(reader, &[]); 
    let mut refs = Vec::new();
    let mut capabilities = Capabilities::default();
    let mut head = None;
    let mut saw_service = false;

    while let Some(result) = packets.read_line() {
        match result {
            Ok(Ok(packet)) => match packet {
            PacketLineRef::Data(data) => {
                let line = std::str::from_utf8(data).unwrap_or("").trim_end();

                // Skip the first line: "# service=git-upload-pack"
                if !saw_service && line.starts_with("# service=") {
                    saw_service = true;
                    continue;
                }

                let (left, right) = line.split_once('\0').unwrap_or((line, ""));
                let mut parts = left.splitn(2, ' ');

                let hash = parts.next().unwrap_or("").to_string();
                let name = parts.next().unwrap_or("").to_string();

                // Save HEAD hash for symbolic resolution
                if name == "HEAD" {
                    head = Some(hash.clone());
                }

                refs.push(AdvertisedRef { hash, name });

                // Only the first line may include capabilities
                if !right.is_empty() {
                    let cap_strings: Vec<&str> = right.split_whitespace().collect();
                    capabilities = parse_capabilities(&cap_strings);
                }
            }
            _ => {}
            },
            Ok(Err(decode_err)) => {
                eprintln!("Decode error: {decode_err}");
                break;
            }
            Err(io_err) => {

                eprintln!("I/O error: {io_err}");
                return Err(io_err);
            }
        }
    }

    Ok(RefAdvertisement {
        refs,
        capabilities,
        head,
    })
}

#[derive(Debug, Default)]
pub struct Capabilities {
    pub multi_ack: bool,
    pub multi_ack_detailed: bool,
    pub thin_pack: bool,
    pub side_band: bool,
    pub side_band_64k: bool,
    pub ofs_delta: bool,
    pub shallow: bool,
    pub no_progress: bool,
    pub include_tag: bool,
    pub allow_tip_sha1_in_want: bool,
    pub allow_reachable_sha1_in_want: bool,
    pub no_done: bool,

    pub symref: Option<(String, String)>,
    pub agent: Option<String>,
    pub object_format: Option<String>,
    
    pub other: Vec<String>,
}

fn parse_capabilities(cap_strings: &[&str]) -> Capabilities {
    let mut caps = Capabilities::default();

    for cap in cap_strings {
        match *cap {
            "multi_ack" => caps.multi_ack = true,
            "multi_ack_detailed" => caps.multi_ack_detailed = true,
            "thin-pack" => caps.thin_pack = true,
            "side-band" => caps.side_band = true,
            "side-band-64k" => caps.side_band_64k = true,
            "ofs-delta" => caps.ofs_delta = true,
            "shallow" => caps.shallow = true,
            "no-progress" => caps.no_progress = true,
            "include-tag" => caps.include_tag = true,
            "allow-tip-sha1-in-want" => caps.allow_tip_sha1_in_want = true,
            "allow-reachable-sha1-in-want" => caps.allow_reachable_sha1_in_want = true,
            "no-done" => caps.no_done = true,

            _ if cap.starts_with("symref=") => {
                if let Some((from, to)) = cap["symref=".len()..].split_once(':') {
                    caps.symref = Some((from.to_string(), to.to_string()));
                }
            }
            _ if cap.starts_with("agent=") => {
                caps.agent = Some(cap["agent=".len()..].to_string());
            }
            _ if cap.starts_with("object-format=") => {
                caps.object_format = Some(cap["object-format=".len()..].to_string());
            }

            other => caps.other.push(other.to_string()),
        }
    }

    caps
}

pub struct RefAdvertisement {
    pub refs: Vec<AdvertisedRef>,
    pub capabilities: Capabilities,
    pub head: Option<String>,
}

pub struct AdvertisedRef {
    pub hash: String,
    pub name: String,
}

impl RefAdvertisement {
    pub fn print_debug(&self) {
        println!("--- RefAdvertisement ---");

        if let Some(head) = &self.head {
            println!("HEAD: {head}");
        } else {
            println!("HEAD: <none>");
        }

        println!("\nCapabilities:");
        let caps = &self.capabilities;
        println!("  multi_ack: {}", caps.multi_ack);
        println!("  multi_ack_detailed: {}", caps.multi_ack_detailed);
        println!("  thin_pack: {}", caps.thin_pack);
        println!("  side_band: {}", caps.side_band);
        println!("  side_band_64k: {}", caps.side_band_64k);
        println!("  ofs_delta: {}", caps.ofs_delta);
        println!("  shallow: {}", caps.shallow);
        println!("  no_progress: {}", caps.no_progress);
        println!("  include_tag: {}", caps.include_tag);
        println!("  allow_tip_sha1_in_want: {}", caps.allow_tip_sha1_in_want);
        println!("  allow_reachable_sha1_in_want: {}", caps.allow_reachable_sha1_in_want);
        println!("  no_done: {}", caps.no_done);

        if let Some((from, to)) = &caps.symref {
            println!("  symref: {from} → {to}");
        }
        if let Some(agent) = &caps.agent {
            println!("  agent: {agent}");
        }
        if let Some(format) = &caps.object_format {
            println!("  object_format: {format}");
        }

        if !caps.other.is_empty() {
            println!("  other:");
            for o in &caps.other {
                println!("    - {o}");
            }
        }

        println!("\nRefs:");
        for r in &self.refs {
            println!("  {} -> {}", r.name, r.hash);
        }

        println!("-------------------------\n");
    }
}

// pub struct PacketReader<R: BufRead> {
//     inner: StreamingPeekableIter<R>,
//     bytes_read: usize,
// }

// impl<R: BufRead> PacketReader<R> {
//     pub fn new(reader: R) -> Self {
//         Self {
//             inner: StreamingPeekableIter::new(reader, &[]),
//             bytes_read: 0,
//         }
//     }

//     pub fn read_line(&mut self) -> io::Result<Option<PacketLineRef<'_>>> {
//         match self.inner.read_line() {
//             Some(Ok(packet)) => {
//                 self.bytes_read += packet.as_bytes().len();
//                 Ok(Some(packet))
//             }
//             Some(Err(err)) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
//             None => Ok(None),
//         }
//     }

//     pub fn bytes_read(&self) -> usize {
//         self.bytes_read
//     }
// }