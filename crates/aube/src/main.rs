// mimalloc as global allocator on release builds. Cuts linker-phase
// wall time and peak RSS on large installs. Per-thread heaps suit
// rayon work-stealing and tokio's blocking pool. Gated on
// `not(debug_assertions)` so `cargo run` and `cargo test` keep the
// system allocator, which keeps Valgrind, ASAN, and Miri happy.
// `secure` feature skipped. aube's hot path is tarball extraction
// with bounded input, not a sandbox boundary.
#[cfg(all(feature = "mimalloc", not(debug_assertions)))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    std::process::exit(aube::run_from_env());
}
