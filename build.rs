fn main() {
    std::process::Command::new("sh")
        .arg("cargo run -- install")
        .status()
        .expect("Failed to install hooks");
}
