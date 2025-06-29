use std::io::{self, BufReader, Cursor};

use git_packetline::{PacketLineRef};
use crate::clone::{caps::{parse_capabilities, Capabilities}, packet_reader::PacketReader};


pub fn parse_ref_advertisement(bytes: &[u8]) -> std::io::Result<RefAdvertisement> {
    let cursor =    BufReader::new(Cursor::new(bytes));
    let mut reader = PacketReader::new(cursor, bytes.len()); 

    _ = match reader.read_line()? {
        Some(PacketLineRef::Data(data)) => {
            let line = std::str::from_utf8(data).unwrap_or("").trim_end();
            if !line.starts_with("# service=") {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing service header"));
            }
        }
        Some(_) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected service line"));
        }
        None => {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "No data in input"));
        }
    };
    
    let mut refs = Vec::new();
    let mut capabilities = Capabilities::default();
    let mut head = None;

    while let Some(packet) = reader.read_line()? {
        match packet {
            PacketLineRef::Data(data) => {
                let line = std::str::from_utf8(data).unwrap_or("").trim_end();
                let (letf, right) = line.split_once('\0').unwrap_or((line, ""));
                let mut parts = letf.splitn(2, ' ');

                let hash = parts.next().unwrap_or("").to_string();
                let name = parts.next().unwrap_or("").to_string();

                if name == "HEAD" {
                    head = Some(hash.clone());
                }

                refs.push(AdvertisedRef { hash: hash, name: name });
                if !right.is_empty() {
                    let caps = right.split_whitespace().collect();
                    capabilities = parse_capabilities(caps);
                }
            },
            PacketLineRef::Flush => {
                if reader.validate_consumed().is_ok() {
                    break;
                } else {
                    continue;
                }
            },
            _ => {
                return  Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected packet type for ref advertisement parsing"));
            }
        }
    }

    reader.validate_consumed()?;

    Ok(RefAdvertisement {
        refs,
        capabilities,
        head,
    })
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
