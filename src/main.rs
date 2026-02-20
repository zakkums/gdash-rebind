//! kmrebind: kernel-level keyboard to mouse button remapper.

fn main() {
    std::process::exit(kmrebind::cli::run());
}
