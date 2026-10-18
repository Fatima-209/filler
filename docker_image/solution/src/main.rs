use std::io;

fn main() {
    if let Err(error) = filler_robot::run(io::stdin().lock(), io::stdout().lock()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
