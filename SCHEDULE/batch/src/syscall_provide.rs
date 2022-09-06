// #![no_std]
// #![no_main]

use output::println;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("\x1b[34m[kernel] Application exited with code {}\x1b[0m", exit_code);
    super::exit_run_next();
}

pub fn sys_yield(exit_code: i32) -> ! {
    // info!("[kernel] Applications suspend with code {}", exit_code);
    println!("\x1b[34m[kernel] Kernel suspend with code {}]\x1b[0m", exit_code);
    super::suspend_run_next();
}