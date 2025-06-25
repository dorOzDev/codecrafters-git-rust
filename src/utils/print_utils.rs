
pub fn print_hex_dump(label: &str, data: &[u8]) {
    println!("--- {} ---", label);
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}: ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        for _ in chunk.len()..16 {
            print!("   ");
        }
        print!(" ");
        for byte in chunk {
            let ch = *byte as char;
            if ch.is_ascii_graphic() || ch == ' ' {
                print!("{}", ch);
            } else {
                print!(".");
            }
        }
        println!();
    }
    println!("--- End of {} ---", label);
}


pub fn print_raw_bytes(raw: &Vec<u8>) {
    for byte in raw {
        print!("{:02x} ", byte);
    }
    println!();
}