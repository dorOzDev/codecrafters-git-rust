use std::io::{self, Result};
use bytes::Bytes;
use reqwest::blocking::Response;



pub fn unpack_pkt_res(res: Response) -> Result<()> {
    if !res.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("unable to unpack response, response status is: {}", res.status())))
    }

    let bytes = res.bytes().map_err(|e| {io::Error::new(io::ErrorKind::Other, format!("failed to read response: {}", e))})?;
    print_pkt_lines_until_pack(&bytes);
    Ok(())
}

pub fn print_pkt_lines_until_pack(bytes: &Bytes) -> io::Result<(usize)> {
    println!("--- Git Fetch Response (text lines before PACK) ---");

    // Find the offset where "PACK" begins
    let pack_start = bytes
        .windows(4)
        .position(|w| w == b"PACK")
        .unwrap_or(bytes.len());

    let mut i = 0;
    while i + 4 <= pack_start {
        // Read pkt-line length
        let len_bytes = &bytes[i..i + 4];
        let len_str = std::str::from_utf8(len_bytes).unwrap_or("0000");
        let len = usize::from_str_radix(len_str, 16).unwrap_or(0);

        if len == 0 {
            println!("[{:04x}] FLUSH", i);
            i += 4;
            continue;
        }

        if i + len > pack_start {
            println!(
                "⚠️  Truncated pkt-line at offset {}: length {}, PACK starts at {}",
                i, len, pack_start
            );
            break;
        }

        let content = &bytes[i + 4..i + len];
        match std::str::from_utf8(content) {
            Ok(text) => println!("[{:04x}] {}", i, text.trim_end()),
            Err(e) => println!(
                "[{:04x}] ⚠️ Invalid UTF-8 at offset {}: {}",
                i, i + 4,
                hex::encode(content)
            ),
        }

        i += len;
    }

    println!("--- End of text section, PACK starts at offset: {:#x} ---", pack_start);
    Ok(pack_start)
}