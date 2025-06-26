use std::io::Cursor;

use git_packetline::{PacketLineRef, StreamingPeekableIter};


pub fn parse_ref_advertisement(bytes: &[u8]) -> std::io::Result<()> {
    let reader = Cursor::new(bytes);
    let mut packets = StreamingPeekableIter::new(reader, &[]); 

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
