use std::{fs, path::Path};
fn visit(p: &Path, files: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(p).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            visit(&p, files)
        } else {
            files.push(p)
        }
    }
}
fn main() {
    let mut files = vec!["Cargo.toml".into(), "Cargo.lock".into(), "build.rs".into()];
    visit(Path::new("src"), &mut files);
    files.sort();
    let mut hash = 0xcbf29ce484222325u64;
    for f in files {
        println!("cargo:rerun-if-changed={}", f.display());
        for b in f
            .to_string_lossy()
            .replace('\\', "/")
            .bytes()
            .chain(fs::read(&f).unwrap())
        {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    println!("cargo:rustc-env=KERR_SOURCE_FINGERPRINT={hash:016x}");
}
