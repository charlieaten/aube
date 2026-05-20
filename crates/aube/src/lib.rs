// The library target exists so hosts can embed Aube without invoking the CLI
// binary. It reuses CLI command modules internally, so some command-only code
// is intentionally unused when the crate is compiled as a library.
#![allow(dead_code)]

mod cli_args;
mod commands;
mod dep_chain;
mod deprecations;
mod dirs;
pub mod embedded;
mod engines;
mod patches;
mod pnpmfile;
mod progress;
mod state;
mod update_check;
mod version;
