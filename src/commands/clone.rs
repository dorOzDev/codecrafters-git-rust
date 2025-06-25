use std::{io::{self, Cursor, Error}, path::PathBuf};

use git_packetline::{PacketLineRef, StreamingPeekableIter};
use reqwest::{blocking::Client, header::USER_AGENT};


pub fn run(args: &[String]) -> io::Result<()> {
    let clone_args = parse_args(&args)?;
    let refs_bytes = fetch_refs(&clone_args.url)?;
    println!("refs bytes len:{}", refs_bytes.len());
    parse_with_git_packetline(&refs_bytes)?;
    println!("Running git-clone with args: {},{}", clone_args.url, clone_args.target_dir.display());
    Ok(())
}

pub struct CloneArgs {
    pub url: String,
    pub target_dir: PathBuf,
}

fn parse_args(args: &[String]) -> io::Result<CloneArgs> {
    if args.is_empty() {
        return Err(Error::new(io::ErrorKind::InvalidInput, "Usage: git clone <url> [directory]"));
    }

    let url = args[0].clone();
    let target_dir = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        let trimmed = url.trim_end_matches(".git");
        let name = trimmed
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("repo");
        PathBuf::from(name)
    };
    Ok(CloneArgs { url, target_dir })
}

fn fetch_refs(url: &str) -> Result<Vec<u8>, std::io::Error> {
    let service_url = format!("{}/info/refs?service=git-upload-pack", url.trim_end_matches('/'));

    let client = Client::new();
    let response = client
        .get(&service_url)
        .header(USER_AGENT, "git/2.42.0")
        .send()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if !response.status().is_success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Request failed with status {}", response.status()),
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(bytes.to_vec())
}

fn parse_with_git_packetline(bytes: &[u8]) -> std::io::Result<()> {
    let reader = Cursor::new(bytes);
    let mut packets = StreamingPeekableIter::new(reader, &[]); // no stopping condition

        while let Some(result) = packets.read_line() {
            match result {
                Ok(Ok(packet)) => match packet {
                    PacketLineRef::Flush => println!("<flush>"),
                    PacketLineRef::Delimiter => println!("<delimiter>"),
                    PacketLineRef::ResponseEnd => {
                        println!("<response-end>");
                        break;
                    }
                    PacketLineRef::Data(data) => {
                        match std::str::from_utf8(data) {
                            Ok(s) => println!("DATA: {s}"),
                            Err(_) => println!("(binary data) {:?}", data),
                        }
                    }
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

    Ok(())

}