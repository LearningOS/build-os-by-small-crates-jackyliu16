#![no_std]
#![no_main]
#![feature(panic_info_message)]
/// the configuration crate of
/// 1. static variable
/// 2. UPSafeCell<T>
/// 3. TrapContext
/// 4. panic_handler
/// 5. TaskContext


/// static variable configuration 
pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;


/// Trap Context
use riscv::register::sstatus::{self, Sstatus, SPP};
// the information when trap we need to keep 
#[repr(C)]
pub struct TrapContext {
    // we will save all register when we trap
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    // last i-addr 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}
/// UPSafeCell<T> configuration 
use core::marker::Sync;
use core::cell::{RefCell, RefMut};

/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.
pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

// use sbi_rt::{system_reset, RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE};
// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_SYSTEM_FAILURE);
//     unreachable!()
// }

use sbi_rt::{self, RESET_REASON_NO_REASON,RESET_TYPE_SHUTDOWN};
use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        output::println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        output::println!("Panicked: {}", info.message().unwrap());
    }
    sbi_rt::system_reset(RESET_TYPE_SHUTDOWN, RESET_REASON_NO_REASON);
    unreachable!()
}