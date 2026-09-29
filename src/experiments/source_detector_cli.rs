//! Thin CLI around the editable experiment and independent postprocessor.
use super::source_detector::{self, LaunchDirection, SourceDetectorExperiment, SourcePattern};
use crate::{
    detector::binning::TimeBasis,
    output::{OutputResult, time_frames::PostprocessConfig},
};
use std::{fs::File, path::PathBuf};
fn vector(s: &str) -> Result<[f64; 3], String> {
    let v: Vec<f64> = s
        .split(',')
        .map(|s| s.parse::<f64>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    v.try_into().map_err(|_| "expected x,y,z".into())
}
fn resolution(s: &str) -> Result<[usize; 2], String> {
    let v: Vec<usize> = s
        .split('x')
        .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    v.try_into().map_err(|_| "expected WIDTHxHEIGHT".into())
}
pub fn run(args: &[String], rebinner: bool) -> OutputResult<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Kerr 3+1 source/detector data\n\
experiment source_to_detector [--config experiment.json] [options]\n\
rebin --input results/source_to_detector --output results/rebinned --dt 0.1\n\
Physical options: --spin N --nu N --nv N --rays SQUARE_COUNT\n\
  --source-center x,y,z --detector-center x,y,z --direction x,y,z\n\
  --local-direction n_r,n_theta,n_phi --t-emit N\n\
  --rtol N --atol N --max-step N --max-steps N --max-affine N --escape N --epsilon N\n\
Postprocess: --dt N --time-start N --time-end N --time-basis t_hit|delta_t\n\
  --resolution WIDTHxHEIGHT --fps N --counts-per-white N --output DIRECTORY\n\
Defaults: 16x16 source, simultaneous t_emit=0, 128x128 detector, dt=1 M.\n\
Rebin reads existing events only. Existing run/event/output files are never overwritten.\n\
Edit experiments/source_detector.rs or a full JSON config for plane bases, sizes and explicit emissions."
        );
        return Ok(());
    }
    let mut exp = SourceDetectorExperiment::default();
    if !rebinner && let Some(i) = args.iter().position(|s| s == "--config") {
        exp = serde_json::from_reader(File::open(args.get(i + 1).ok_or("missing config path")?)?)?;
    }
    let mut post = if rebinner {
        PostprocessConfig::default()
    } else {
        exp.postprocess.clone()
    };
    let mut input = PathBuf::from("results/source_to_detector");
    let mut output = None;
    let mut res = None;
    let mut i = 0;
    while i < args.len() {
        let key = &args[i];
        let v = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {key}"))?;
        match key.as_str() {
            "--config" if !rebinner => {}
            "--input" if rebinner => input = PathBuf::from(v),
            "--output" => output = Some(PathBuf::from(v)),
            "--dt" => post.time_bin_width = v.parse()?,
            "--time-start" => post.start = Some(v.parse()?),
            "--time-end" => post.end = Some(v.parse()?),
            "--time-basis" => {
                post.basis = match v.as_str() {
                    "t_hit" => TimeBasis::Arrival,
                    "delta_t" => TimeBasis::Travel,
                    _ => return Err("time basis is t_hit or delta_t".into()),
                }
            }
            "--resolution" => res = Some(resolution(v)?),
            "--fps" => post.playback_fps = v.parse()?,
            "--counts-per-white" => post.counts_per_white = v.parse()?,
            "--spin" if !rebinner => exp.spin = v.parse()?,
            "--nu" | "--nv" if !rebinner => {
                let (mut nu, mut nv) = match exp.source.pattern {
                    SourcePattern::RectangularGrid { nu, nv } => (nu, nv),
                    SourcePattern::SquareGrid { side } => (side, side),
                    _ => (1, 1),
                };
                if key == "--nu" {
                    nu = v.parse()?;
                } else {
                    nv = v.parse()?;
                }
                exp.source.pattern = SourcePattern::RectangularGrid { nu, nv };
            }
            "--rays" if !rebinner => {
                let count: usize = v.parse()?;
                let side = count.isqrt();
                if side * side != count {
                    return Err(
                        "--rays needs a square count; use --nu and --nv for a rectangle".into(),
                    );
                }
                exp.source.pattern = SourcePattern::SquareGrid { side };
            }
            "--source-center" if !rebinner => exp.source.plane.center = vector(v)?,
            "--detector-center" if !rebinner => exp.detector.plane.center = vector(v)?,
            "--direction" if !rebinner => {
                exp.source.direction = LaunchDirection::CartesianSpatial(vector(v)?)
            }
            "--local-direction" if !rebinner => {
                exp.source.direction = LaunchDirection::LocalZamo(vector(v)?)
            }
            "--t-emit" if !rebinner => exp.source.t_emit = v.parse()?,
            "--rtol" if !rebinner => exp.integration.rtol = v.parse()?,
            "--atol" if !rebinner => exp.integration.atol = v.parse()?,
            "--max-step" if !rebinner => exp.integration.max_step = v.parse()?,
            "--max-steps" if !rebinner => exp.integration.max_steps = v.parse()?,
            "--max-affine" if !rebinner => exp.integration.max_affine = v.parse()?,
            "--epsilon" if !rebinner => exp.integration.horizon_epsilon = v.parse()?,
            "--escape" if !rebinner => exp.integration.escape_radius = v.parse()?,
            _ => return Err(format!("unknown/inapplicable option {key}; use --help").into()),
        }
        i += 2;
    }
    if rebinner {
        let output = output.unwrap_or_else(|| {
            input.join(format!(
                "rebin_{}",
                (crate::output::metadata::unix_seconds() * 1000.0) as u64
            ))
        });
        source_detector::rebin(&input, &output, &post, res)?;
    } else {
        exp.postprocess = post;
        if let Some(r) = res {
            exp.detector.resolution = r;
        }
        if let Some(o) = output {
            exp.output = o;
        }
        source_detector::simulate(&exp)?;
    }
    Ok(())
}
