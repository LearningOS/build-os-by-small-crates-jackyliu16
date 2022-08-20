#![no_std]
#![no_main]

mod task_manager;
mod syscall_provide;
use riscv::register::{stvec, utvec::TrapMode};
pub use syscall_provide::{sys_exit, sys_yield};

use basic::*;
use output::*;
use output::log::*;
use loader::*;

extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`.
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}

use task_manager::TASK_MANAGER;
// 由于尝试将`run_first_task and run_next_task` 合并的操作失败了，因此没有办法简单的的使用
pub fn init() {
    info!("using init funciton from time-sharing");
    TASK_MANAGER.run_first_task();
}

use task_manager::*;
pub fn suspend_run_next() {
    TASK_MANAGER.suspend_current_task();
    TASK_MANAGER.run_next_task();
}

pub fn exit_run_next() {
    info!("exit and run next application");
    TASK_MANAGER.exit_current_task();
    TASK_MANAGER.run_next_task();
}

pub use syscall_provide::*;