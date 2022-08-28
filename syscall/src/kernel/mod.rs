
mod fs;
use fs::sys_write;
use super::SyscallId;
use config::SCHEDULE::*;
#[allow(unused_imports)]
use output::log::*;

/// syscall handler[这个地方本来想用[usizel; 6]的但是不行]
pub fn syscall_handler(syscall_id: SyscallId, args: [usize; 3]) -> isize {
    // debug!("sysacall_id: {} args:{:?}", syscall_id.0, args);
    match syscall_id {
        SyscallId::WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SyscallId::EXIT => sys_exit(args[0] as i32),
        SyscallId::SCHED_YIELD => sys_yield(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id.0),
    };
    0
}