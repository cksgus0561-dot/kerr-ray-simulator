//! Primary events are append-once data. Every writer uses create_new, never truncates.
use super::OutputResult;
use crate::detector::events::HitEvent;
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
};
pub fn new_file(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}
pub fn read_events(path: &Path) -> OutputResult<Vec<HitEvent>> {
    let mut events = Vec::new();
    for row in ::csv::Reader::from_path(path)?.deserialize() {
        let e: HitEvent = row?;
        e.validate()?;
        events.push(e);
    }
    Ok(events)
}
pub fn counts(path: &Path, data: &[u64], width: usize) -> OutputResult<()> {
    let mut w = BufWriter::new(new_file(path)?);
    for row in data.chunks(width) {
        for (i, c) in row.iter().enumerate() {
            if i > 0 {
                write!(w, ",")?;
            }
            write!(w, "{c}")?;
        }
        writeln!(w)?;
    }
    w.flush()?;
    Ok(())
}
