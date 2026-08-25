fn main() {
    if let Err(error) = pseudo3d::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
