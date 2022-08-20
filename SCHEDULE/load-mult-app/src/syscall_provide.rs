// #![no_std]
// #![no_main]

use output::log::*;

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    super::run_next_app();
}