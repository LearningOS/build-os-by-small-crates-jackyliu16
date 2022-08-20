use output::log::*;
use crate::stack::*;
use basic::*;

/// Get base address of app i.
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

/// Get the total number of applications.
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// Load nth user app at
/// [APP_BASE_ADDRESS + n * APP_SIZE_LIMIT, APP_BASE_ADDRESS + (n+1) * APP_SIZE_LIMIT).
#[deprecated]
fn load_all_apps() {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // clear i-cache first
    unsafe {
        core::arch::asm!("fence.i");
    }
    // load apps
    for i in 0..num_app {
        // base addr of appi
        let base_i = get_base_i(i);
        // clear region
        // use iter as ptr to clear memory
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // load app from data section to memory
        // src is data of appi, which is stored in [app_start[i], app_start[i+1]).
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        // dst is ptr of memory space going to be loaded. 
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}

/// get app info with entry and sp and save `TrapContext` in kernel stack
pub fn init_app_cx(app_id: usize) -> usize {
    /* sp point to the first instruction, which is base_addr */
    KERNEL_STACK[app_id].push_context(
        TrapContext::app_init_context(
        get_base_i(app_id), 
        USER_STACK[app_id].get_sp(),
    ))
}

pub unsafe fn load_app(start_address: usize, end_address: usize, load_pos: usize) {
    debug!("trying to load the context of [{:#X}, {:#X}] to [{:#X}, {:#X}]",
    start_address,
        end_address,
        load_pos,
        load_pos + (end_address - start_address)    
    );
    // clear icache and loading space
    core::arch::asm!("fence.i");
    core::slice::from_raw_parts_mut(load_pos as *mut u8, APP_SIZE_LIMIT).fill(0);

    let app_src = core::slice::from_raw_parts(
        start_address as *const u8,
        end_address - start_address,
    );
    let app_dst = core::slice::from_raw_parts_mut(
        load_pos as *mut u8,
        app_src.len()
    );
    app_dst.copy_from_slice(app_src);
}