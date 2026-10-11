use std::process::Command;
extern crate embed_resource;

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
}

fn main() {
    println!("cargo:rerun-if-changed=src/build.rs");
    println!("cargo:rerun-if-changed=windows");

    embed_resource::compile("windows/resources.rc", embed_resource::NONE).manifest_optional().unwrap();

    for entry in ["HEAD", "refs", "packed-refs"] {
        if let Some(path) = git_output(&["rev-parse", "--git-path", entry]) {
            if std::path::Path::new(&path).exists() {
                println!("cargo:rerun-if-changed={path}");
            }
        }
    }

    let git_hash = match git_output(&["tag", "--points-at", "HEAD"]) {
        Some(tags) if tags.is_empty() => git_output(&["rev-parse", "--short", "HEAD"]).unwrap_or_default(),
        _ => String::new(),
    };
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
