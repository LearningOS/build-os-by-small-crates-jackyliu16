#![no_std]
#![no_main]
#![feature(panic_info_message)]
/// the configuration crate of
/// 1. static variable
/// 2. UPSafeCell<T>
/// 3. TrapContext
/// 4. panic_handler


/// static variable configuration 
pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;


mod up;
pub use up::UPSafeCell;

mod loader;
pub use loader::*;

mod trap;
pub use trap::*;

mod panic;
pub use panic::*;