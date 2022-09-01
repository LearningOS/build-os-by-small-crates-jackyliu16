#![no_std]
#![no_main]
#![allow(non_snake_case)]

/// trait SCHEDULE {
///     pub fn init(),
///     // pub fn run_next(),
///     pub fn suspend_run_next(),
///     pub fn exit_run_next(),
/// }

mod config;
mod app_manager;
mod syscall_provide;

#[allow(unused)]
#[macro_use]
use output::log::*;
use app_manager::TASK_MANAGER;

// 基于我的设想，这三种不同的调用是提供给所有对象的，所有对象都可以使用这三种操作。
// 针对于系统调用访民啊的操作，主要基于
pub fn init() { 
    info!("using init function from time-sharing");
    TASK_MANAGER.print_app_info();
    TASK_MANAGER.run_first_task();
}

pub fn suspend_run_next() -> ! {
    info!("lib.rs: suspend and run next app");
    TASK_MANAGER.suspend_current_task();
    TASK_MANAGER.run_next_task()
}

pub fn exit_run_next() -> ! {
    info!("exit and run next application:");
    TASK_MANAGER.exited_current_task();
    TASK_MANAGER.run_next_task()
}

// 向外提供的系统调用
pub use syscall_provide::*;
