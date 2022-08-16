use output::*;
use output::log::*;

const FD_STDOUT: usize = 0; // 原来的是1，但是我用户态程序那边似乎有问题因此没有办法正常运行，只能出此下策

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // debug!("[KERNEL] sys_write: fd: {}", fd);
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            error!("unsupported");
            panic!("Unsupported fd in sys_write!");
        }
    }
}
