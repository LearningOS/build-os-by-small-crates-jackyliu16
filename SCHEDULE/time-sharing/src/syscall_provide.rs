// #![no_std]
// #![no_main]

use output::log::*;

pub fn sys_exit(exit_code: i32) -> ! {
    // info!("[kernel] Application exited with code {}", exit_code);
    super::exit_run_next();
}

pub fn sys_yield(exit_code: i32) -> ! {
    // info!("[kernel] Applications suspend with code {}", exit_code);
    super::suspend_run_next();
}