#[cfg(not(windows))]
compile_error!("This crate is Windows-only for now");

pub mod process;
pub mod module;
pub mod errors;
