#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![deny(warnings)]
#![allow(non_snake_case)]

use sbi_rt::*;
use output::log::*;
use config::SCHEDULE;

// 内联app.asm 进到程序中来
core::arch::global_asm!(include_str!(env!("APP_ASM")));     
core::arch::global_asm!(include_str!(env!("TRAP")));       
core::arch::global_asm!(include_str!(env!("SWITCH")));


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
    extern "C" {
        static mut sbss: u64;
        static mut ebss: u64;
    }
    unsafe { r0::zero_bss(&mut sbss, &mut ebss) };      
}

extern "C" fn primary_rust_main() -> ! {
    // println!("ch: {}", env!("CH"));

    clear_bss();
    output::init_console(&Console);
    // output::set_log_level(option_env!("Info"));
    set_max_level(LevelFilter::Debug);
    
    error!("[KERNEL] you are now inside the main function");
    trace!("[KERNEL] you are now inside the main function");
    info!("[KERNEL] you are now inside the main function");
    warn!("[KERNEL] you are now inside the main function");
    debug!("[KERNEL] you are now inside the main function");
    
    trap::init();   // init stvec
    SCHEDULE::init();

    system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    unreachable!()
}


// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
//     unreachable!()
// }

struct Console;

/// 为 `Console` 实现 `output::Console` trait。
impl output::Console for Console {
    fn put_char(&self, c: u8) {
        #[allow(deprecated)]
        legacy::console_putchar(c as _);
    }
}
