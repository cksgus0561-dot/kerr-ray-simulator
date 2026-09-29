//! Double-clickable native entry point. CLI options are identical to `visualize`.
//! An optional kerr-viewer.json beside the executable provides startup settings.
fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty()
        && let Ok(executable) = std::env::current_exe()
    {
        // Optional launcher arguments support opening a stored run by double-click,
        // with exactly the same validated CLI path and no new simulation config.
        let launch = executable.with_file_name("kerr-viewer.args.json");
        if launch.is_file() {
            args = serde_json::from_slice(&std::fs::read(launch).expect("read viewer arguments"))
                .expect("viewer args must be a JSON string array");
        }
        let path = executable.with_file_name("kerr-viewer.json");
        if args.is_empty() && path.is_file() {
            args.extend(["--config".into(), path.to_string_lossy().into_owned()]);
        }
    }
    if let Err(error) = kerr_ray::visualization::cli::run(&args, false) {
        eprintln!("Kerr visualization failed: {error}");
        std::process::exit(1);
    }
}
