//! 16-bit grayscale derived only from histogram counts. No text, axes, spin or smoothing.
//! Fixed counts_per_white across every output frame, explicitly recorded in metadata.
use super::{OutputResult, csv::new_file};
use std::{fs::File, io::BufWriter, path::Path};
pub fn pixels(counts: &[u64], counts_per_white: u64) -> Vec<u8> {
    counts
        .iter()
        .flat_map(|c| {
            (((*c).min(counts_per_white) as u128 * 65535 / counts_per_white as u128) as u16)
                .to_be_bytes()
        })
        .collect()
}
pub fn encoder(
    path: &Path,
    res: [usize; 2],
    frames: Option<usize>,
    fps: u16,
) -> OutputResult<png::Writer<BufWriter<File>>> {
    let mut e = png::Encoder::new(
        BufWriter::new(new_file(path)?),
        res[0] as u32,
        res[1] as u32,
    );
    e.set_color(png::ColorType::Grayscale);
    e.set_depth(png::BitDepth::Sixteen);
    if let Some(n) = frames {
        e.set_animated(n as u32, 0)?;
        e.set_frame_delay(1, fps)?;
    }
    Ok(e.write_header()?)
}
pub fn write(path: &Path, res: [usize; 2], data: &[u8]) -> OutputResult<()> {
    let mut w = encoder(path, res, None, 30)?;
    w.write_image_data(data)?;
    w.finish()?;
    Ok(())
}
