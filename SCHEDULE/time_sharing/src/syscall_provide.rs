
use super::*;

pub fn sys_exit(exit_code: i32) -> isize {
    exit_run_next();
    0
}

pub fn sys_yield() -> isize {
    suspend_run_next();
    0
}