use std::{path::PathBuf, process::Command};


const TCC_PATH: PathBuf = "/home/jasper/Downloads/tcc-0.9.26/fuck";

pub fn run_script(path: &str) {
    let out_dir = "./fuck";

    Command::new("tcc")
        .args([path, "-o", out_dir]);
}