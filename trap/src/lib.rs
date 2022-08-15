#![no_std]
#![no_main]

use basic::*;
use config::SCHEDULE::{exit_run_next};
use syscall::kernel::syscall_handler;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};
use sbi_rt::{system_reset, RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE};
use output::log::*;


/// 修改 stvec 寄存器来指向正确的 Trap 处理入口点
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        // init the stvec for init trap register
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// Trap 处理的总体流程如下：
/// 首先通过 __alltraps 将 Trap 上下文保存在内核栈上，
/// 然后跳转到使用 Rust 编写的 trap_handler 函数完成 Trap 分发及处理。
/// 当 trap_handler 返回之后，使用 __restore 从保存在内核栈上的 Trap 上下文恢复寄存器。
/// 最后通过一条 sret 指令回到应用程序执行。
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    info!("inside trap handler");
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            info!("using environment call");
            cx.sepc += 4;
            cx.x[10] = syscall_handler(cx.x[17].into(), [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            error!("[kernel] PageFault in application, core dumped.");
            exit_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application, core dumped.");
            exit_run_next();
        }
        _ => {
            error!("unsupported trap");
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}