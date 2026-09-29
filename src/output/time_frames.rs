//! Streaming histogram -> PNG sequences and APNG. Space*time dense arrays are avoided.
use super::{OutputResult, accumulated_image as image, csv, metadata};
use crate::detector::{
    binning::{TimeBasis, TimeBins, bin},
    events::HitEvent,
    plane::DetectorPlane,
};
use serde_json::json;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PostprocessConfig {
    pub time_bin_width: f64,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub basis: TimeBasis,
    /// Playback FPS only affects APNG frame delays. It has no physical meaning.
    pub playback_fps: u16,
    pub counts_per_white: u64,
}
impl Default for PostprocessConfig {
    fn default() -> Self {
        Self {
            time_bin_width: 1.0,
            start: None,
            end: None,
            basis: TimeBasis::Arrival,
            playback_fps: 30,
            counts_per_white: 4,
        }
    }
}
impl PostprocessConfig {
    pub fn resolve(&self, events: &[HitEvent]) -> Result<TimeBins, String> {
        if self.playback_fps == 0 || self.playback_fps > 240 || self.counts_per_white == 0 {
            return Err("FPS must be 1..240 and counts_per_white>0".into());
        }
        let time = |e: &HitEvent| match self.basis {
            TimeBasis::Arrival => e.t_hit,
            TimeBasis::Travel => e.delta_t,
        };
        let min = events
            .iter()
            .map(time)
            .min_by(f64::total_cmp)
            .unwrap_or(0.0);
        let max = events
            .iter()
            .map(time)
            .max_by(f64::total_cmp)
            .unwrap_or(min);
        let dt = self.time_bin_width;
        let start = self.start.unwrap_or((min / dt).floor() * dt);
        let end = self
            .end
            .unwrap_or(start + (((max - start) / dt).floor().max(0.0) + 1.0) * dt);
        let bins = TimeBins {
            width: dt,
            start,
            end,
            basis: self.basis,
        };
        bins.count()?;
        Ok(bins)
    }
}
pub fn generate(
    events_path: &Path,
    events: &[HitEvent],
    det: &DetectorPlane,
    cfg: &PostprocessConfig,
    dir: &Path,
) -> OutputResult<serde_json::Value> {
    let time = cfg.resolve(events)?;
    let b = bin(events, det, time)?;
    let frames = b.by_bin.len();
    let pixels = det.resolution[0] * det.resolution[1];
    // Explicit resource limit; change window/resolution rather than silently dropping events.
    if frames.checked_mul(pixels).is_none_or(|n| n > 500_000_000) {
        return Err(
            "derived output exceeds 500M pixels per mode; reduce resolution or time window".into(),
        );
    }
    fs::create_dir_all(dir)?;
    csv::counts(
        &dir.join("detector_accumulated.csv"),
        &b.accumulated,
        det.resolution[0],
    )?;
    csv::counts(
        &dir.join("detector_before_window.csv"),
        &b.before_window,
        det.resolution[0],
    )?;
    image::write(
        &dir.join("detector_accumulated.png"),
        det.resolution,
        &image::pixels(&b.accumulated, cfg.counts_per_white),
    )?;
    let inst = dir.join("detector_frames/instantaneous");
    let cum = dir.join("detector_frames/cumulative");
    fs::create_dir_all(&inst)?;
    fs::create_dir_all(&cum)?;
    let mut animation_i = image::encoder(
        &dir.join("detector_instantaneous.apng"),
        det.resolution,
        Some(frames),
        cfg.playback_fps,
    )?;
    let mut animation_c = image::encoder(
        &dir.join("detector_cumulative.apng"),
        det.resolution,
        Some(frames),
        cfg.playback_fps,
    )?;
    let mut index = BufWriter::new(csv::new_file(&dir.join("frame_index.csv"))?);
    let mut histogram = BufWriter::new(csv::new_file(&dir.join("detector_time_bins.csv"))?);
    writeln!(
        index,
        "frame,time_start,time_end,new_count,cumulative_count"
    )?;
    writeln!(histogram, "frame,x_pixel,y_pixel,count")?;
    let mut instantaneous = vec![0u64; pixels];
    let mut cumulative = b.before_window.clone();
    for (frame, hits) in b.by_bin.iter().enumerate() {
        instantaneous.fill(0);
        for pixel in hits {
            instantaneous[*pixel] += 1;
            cumulative[*pixel] += 1;
        }
        for (pixel, count) in instantaneous.iter().enumerate() {
            if *count > 0 {
                writeln!(
                    histogram,
                    "{frame},{},{},{count}",
                    pixel % det.resolution[0],
                    pixel / det.resolution[0]
                )?;
            }
        }
        writeln!(
            index,
            "{frame},{:.17e},{:.17e},{},{}",
            time.start + frame as f64 * time.width,
            (time.start + (frame + 1) as f64 * time.width).min(time.end),
            hits.len(),
            cumulative.iter().sum::<u64>()
        )?;
        let pi = image::pixels(&instantaneous, cfg.counts_per_white);
        let pc = image::pixels(&cumulative, cfg.counts_per_white);
        image::write(
            &inst.join(format!("frame_{frame:06}.png")),
            det.resolution,
            &pi,
        )?;
        image::write(
            &cum.join(format!("frame_{frame:06}.png")),
            det.resolution,
            &pc,
        )?;
        animation_i.write_image_data(&pi)?;
        animation_c.write_image_data(&pc)?;
    }
    animation_i.finish()?;
    animation_c.finish()?;
    index.flush()?;
    histogram.flush()?;
    let info = json!({"schema_version":1,"source_events":events_path.canonicalize()?.to_string_lossy(),
        "source_events_fnv1a64":metadata::fingerprint(events_path)?,"time_bins":time,"postprocess":cfg,
        "resolution":det.resolution,"frame_count":frames,"event_count":events.len(),"inside_detector_count":b.spatial_count,
        "analysis_window_count":b.window_count,"before_window_count":b.before_window.iter().sum::<u64>(),"after_window_count":b.after_window,
        "outside_detector_count":b.outside_detector,"time_interval":"[start+k*width, min(end,start+(k+1)*width))",
        "cumulative_convention":"includes all events before each bin end, including before analysis start",
        "array_layout":"row-major; x increases with u; row 0 is +v; absent sparse cells are zero",
        "image_mapping":"16-bit grayscale: round down 65535 * min(count,counts_per_white)/counts_per_white; no labels/axes",
        "floating_edge_rule":"bin index within 4*EPSILON*max(1,abs(index)) of an integer snaps to that edge; event times unchanged",
        "physical_time":"f64 Boyer-Lindquist t or t_hit-t_emit in G=c=M=1; APNG FPS is playback only"});
    metadata::write(&dir.join("derived_metadata.json"), &info)?;
    Ok(info)
}
