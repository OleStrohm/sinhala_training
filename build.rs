use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=dictionary.txt");
    println!("cargo:rerun-if-changed=hooks/slim_down_fonts.sh");

    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() != "wasm" {
        assert!(
            Command::new("bash")
                .args(["./hooks/slim_down_fonts.sh"])
                .status()
                .expect("Could not slim down fonts")
                .success()
        );
    }
}
