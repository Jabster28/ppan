use std::process::{exit, Command};
use std::str;

static CARGOENV: &str = "cargo:rustc-env=";

fn main() {
    let tag = match Command::new("git")
        .args(["describe", "--tags", "--abbrev=0", "--always"])
        .output()
    {
        Ok(output) => str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .trim()
            .to_string(),
        Err(_) => {
            eprintln!("Failed to get git tag");
            exit(1);
        }
    };
    println!("{}CURRENT_TAG={}", CARGOENV, tag);
}
