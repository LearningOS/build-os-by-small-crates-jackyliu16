#![no_std]
#![no_main]
#![feature(panic_info_message)]
/// the configuration crate of
/// 1. static variable
/// 2. UPSafeCell<T>
/// 3. TrapContext
/// 4. panic_handler

mod task;
pub use task::*;

mod upsafecell;
pub use upsafecell::UPSafeCell;

mod trap;
pub use trap::*;

mod panic;
pub use panic::*;

/// static variable configuration 
pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const KERNEL_HEAP_SIZE: usize = 0x20000;
pub const CLOCK_FREQ: usize = 12500000;
pub const MAX_SYSCALL_NUM: usize = 500;


