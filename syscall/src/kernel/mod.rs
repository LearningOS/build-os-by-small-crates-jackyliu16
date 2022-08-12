use super::syscall::SyscallId::*;

mod fs;
use fs::sys_write;
use config::SCHEDULE::sys_exit;

/// syscall handler
pub fn handler(syscall_id: SyscallId, args: [usize; 6]) -> isize {
    match syscall_id {
        WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}