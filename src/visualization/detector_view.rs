//! Histograms use every physical hit, independent of trajectory selection.
use crate::{
    detector::binning::{TimeBins, bin},
    session::{DetectorMode, SessionConfig},
    simulation::SimulationData,
};
pub struct DetectorView {
    pub counts: Vec<u64>,
    pub rgba: Vec<u8>,
    pub total: u64,
    pub displayed: u64,
    pub resolution: [usize; 2],
    pub bin_range: [f64; 2],
    pub hit_filter: [f64; 2],
    config_key: String,
    last_key: (usize, usize, u8),
    entries: Vec<(f64, usize)>,
    accumulated: Vec<u64>,
    by_bin: Vec<Vec<usize>>,
    bins: TimeBins,
}
impl Default for DetectorView {
    fn default() -> Self {
        Self {
            counts: vec![],
            rgba: vec![],
            total: 0,
            displayed: 0,
            resolution: [0, 0],
            bin_range: [0., 1.],
            hit_filter: [0., -1.],
            config_key: String::new(),
            last_key: (usize::MAX, 0, 0),
            entries: vec![],
            accumulated: vec![],
            by_bin: vec![],
            bins: TimeBins {
                width: 1.,
                start: 0.,
                end: 1.,
                basis: crate::detector::binning::TimeBasis::Arrival,
            },
        }
    }
}
impl DetectorView {
    pub fn invalidate(&mut self) {
        self.config_key.clear();
    }
    pub fn update(
        &mut self,
        data: &SimulationData,
        cfg: &SessionConfig,
        time: f64,
    ) -> Result<bool, String> {
        let key = serde_json::to_string(&(
            cfg.experiment.detector.resolution,
            &cfg.experiment.postprocess,
        ))
        .map_err(|e| e.to_string())?;
        let rebuild = key != self.config_key;
        if rebuild {
            let mut det = data.experiment.detector.clone();
            det.resolution = cfg.experiment.detector.resolution;
            det.validate(crate::physics::kerr::Kerr::new(data.experiment.spin)?)?;
            self.bins = cfg.experiment.postprocess.resolve(&data.events)?;
            let b = bin(&data.events, &det, self.bins)?;
            self.accumulated = b.accumulated;
            self.by_bin = b.by_bin;
            self.total = b.spatial_count;
            self.resolution = det.resolution;
            self.counts.resize(self.accumulated.len(), 0);
            self.rgba.resize(self.accumulated.len() * 4, 0);
            self.entries = data
                .events
                .iter()
                .filter_map(|e| {
                    det.pixel([e.u_hit, e.v_hit])
                        .map(|[x, y]| (self.bins.time(e), y * det.resolution[0] + x))
                })
                .collect();
            self.entries.sort_by(|a, b| a.0.total_cmp(&b.0));
            self.config_key = key;
        }
        self.hit_filter = [
            self.entries.partition_point(|e| e.0 <= time) as f64,
            self.bins.index(time).map_or(-1., |i| i as f64),
        ];
        let (start, end, mode) = match cfg.view.detector_mode {
            DetectorMode::Accumulated => (0, self.entries.len(), 0),
            DetectorMode::Cumulative => (0, self.entries.partition_point(|e| e.0 <= time), 2),
            DetectorMode::Instantaneous => {
                if let Some(index) = self.bins.index(time) {
                    let lo = self.bins.start + index as f64 * self.bins.width;
                    let hi = self.bins.end.min(lo + self.bins.width);
                    self.bin_range = [lo, hi];
                    (index, index + 1, 1)
                } else {
                    self.bin_range = [time, time];
                    (0, 0, 1)
                }
            }
        };
        if !rebuild && self.last_key == (start, end, mode) {
            return Ok(false);
        }
        self.last_key = (start, end, mode);
        self.counts.fill(0);
        if mode == 0 {
            self.counts.copy_from_slice(&self.accumulated);
        } else if mode == 1 {
            // Reuse the reference bin assignment, including its decimal-edge rule.
            if start < end {
                for &p in &self.by_bin[start] {
                    self.counts[p] += 1;
                }
            }
        } else {
            for &(_, p) in &self.entries[start..end] {
                self.counts[p] += 1;
            }
        }
        self.displayed = self.counts.iter().sum();
        let white = cfg.experiment.postprocess.counts_per_white;
        for (pixel, count) in self
            .rgba
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(&self.counts)
        {
            let value = (255 * (*count).min(white) / white) as u8;
            pixel.copy_from_slice(&[value, value, value, if *count > 0 { 230 } else { 0 }]);
        }
        Ok(true)
    }
}
