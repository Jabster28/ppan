use std::process::{exit, Command};
use std::str;

static CARGOENV: &str = "cargo:rustc-env=";

fn main() {
    let tag = match Command::new("git")
        .args(["describe", "--tags", "--abbrev=0", "--always"])
        .output()
    {
        Ok(output) => str::from_utf8(output.stdout.as_slice())
            .expect("failed to parse git tag")
            .trim()
            .to_string(),
        Err(e) => {
            eprintln!("Failed to get git tag: {e}");
            exit(1);
        }
    };
    println!("{CARGOENV}CURRENT_TAG={tag}");
}
