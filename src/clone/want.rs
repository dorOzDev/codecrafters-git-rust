use std::io::{Result, Write};

use crate::clone::caps::Capabilities;



pub fn send_want<W: Write>(writer: &mut W, want_hash: &str, caps: &Capabilities) -> Result<()> {
    let line = format!("want {} {}", want_hash, caps.to_git_line());
    log::debug!("sent want: {}", line);
    write_pkt_line(writer, &line)?;

    writer.write_all(b"0000")?;
    Ok(())
}

fn write_pkt_line<W: Write>(writer: &mut W, data: &str) -> Result<()> {
    let total_len = 4 + data.len();
    write!(writer, "{:04x}{}", total_len, data)?;
    Ok(())
}
