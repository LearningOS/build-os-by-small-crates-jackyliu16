#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![deny(warnings)]
#![allow(non_snake_case)]

use sbi_rt::*;
// #[macro_use]
use output::log::{error, debug, warn, trace, info};
// use output::{set_log_level, init_console};
// use output::*;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 4096;

    #[link_section = ".bss.uninit"]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::asm!(
        "   la    sp, {stack}
            li    t0, {stack_size}
            add   sp, sp, t0
            j    {main}
        ",
        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym primary_rust_main,
        options(noreturn),
    )
}

fn clear_bss() {
    // extern "C" {
    //     fn sbss();
    //     fn ebss();
    // }
    // unsafe {
    //     core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
    //         .fill(0);
    extern "C" {
        static mut sbss: u64;
        static mut ebss: u64;
    }
    unsafe { r0::zero_bss(&mut sbss, &mut ebss) };      
}

extern "C" fn primary_rust_main() -> ! {
    clear_bss();
    output::init_console(&Console);
    output::set_log_level(option_env!("DEBUG"));
    
    error!("[KERNEL] you are now inside the main function");
    trace!("[KERNEL] you are now inside the main function");
    info!("[KERNEL] you are now inside the main function");
    warn!("[KERNEL] you are now inside the main function");
    debug!("[KERNEL] you are now inside the main function");
    
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    unreachable!()
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
    unreachable!()
}

struct Console;

/// 为 `Console` 实现 `output::Console` trait。
impl output::Console for Console {
    fn put_char(&self, c: u8) {
        #[allow(deprecated)]
        legacy::console_putchar(c as _);
    }
}