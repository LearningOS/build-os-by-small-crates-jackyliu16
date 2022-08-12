#![no_std]
#![no_main]
#![allow(non_snake_case)]

/// trait SCHEDULE {
///     pub fn init(),
///     // pub fn run_next(),
///     pub fn suspend_run_next(),
///     pub fn exit_run_next(),
/// }

mod syscall_provide;
pub use syscall_provide::sys_write;


#[allow(unused)]
#[macro_use]
use output::log::*;

pub fn init() { 
    debug!("using init function from batch");
}
pub fn suspend_run_next() {
    warn!("using suspend_run_next which haven't been realize in batch");
}

pub fn exit_run_next() {
    debug!("exit_run_next");
    panic!("test complete");
}
