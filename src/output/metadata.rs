//! Research provenance: physical settings and time convention stay outside image pixels.
use super::{OutputResult, csv::new_file};
use std::{
    io::{BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
pub fn unix_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
pub fn write(path: &Path, value: &impl serde::Serialize) -> OutputResult<()> {
    let mut w = BufWriter::new(new_file(path)?);
    serde_json::to_writer_pretty(&mut w, value)?;
    writeln!(w)?;
    w.flush()?;
    Ok(())
}
/// Non-cryptographic provenance checksum; not a security signature.
pub fn fingerprint(path: &Path) -> OutputResult<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut h = 0xcbf29ce484222325u64;
    let mut buf = [0; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        for b in &buf[..n] {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    Ok(format!("{h:016x}"))
}
