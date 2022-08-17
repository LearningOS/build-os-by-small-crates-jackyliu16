#![no_std]
#![no_main]

use basic::*;
use output::log::*;

// #[cfg(all(feature = "batch", feature = "user"))]
// panic!("you can only using it in one kinds of operation system");

/// trait LOADER {
///     /// loading the application from base_address to application_size_limit
///     pub fn load_app(start_address: usize, end_address: usize) 
/// 
/// }

#[cfg(feature = "batch")]
pub unsafe fn load_app(start_address: usize, end_address: usize, load_pos: usize) {
    // clear icache
    // we need to clear instruction-cache 
    // to allow running next app properly.  
    core::arch::asm!("fence.i");            // 调用了一个汇编的东西，实现了清除缓存
    // clear app area, set data from APP_BASE_ADDRESS.. to 0
    core::slice::from_raw_parts_mut(load_pos as *mut u8, APP_SIZE_LIMIT).fill(0);   // BC batch we using a same location to load application so this address will be same.
    // copy [app_start..app_end] to [APP_BASE_ADDRESS..] so as to run it
    let app_src = core::slice::from_raw_parts(
        start_address as *const u8, end_address - start_address
    );
    let app_dst = core::slice::from_raw_parts_mut(load_pos as *mut u8, app_src.len());
    debug!("finish APP_MANAGER.load_app");
    app_dst.copy_from_slice(app_src);
}