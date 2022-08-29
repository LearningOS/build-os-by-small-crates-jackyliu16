use crate::config::*;
use lazy_static::*;
use output::log::*;
use basic::*;
use basic::UPSafeCell;

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    // prints total app number, and their start and end addresses
    pub fn print_app_info(&self) {
        info!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            info!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
        // debug!("KERNEL_STACK:{:X}", KERNEL_STACK.get_sp());
        // debug!("USER_STACK:{:X}", USER_STACK.get_sp());
        // BC the different between kernel stack and user stack is 0x1000, which indicate we alloc correct.
    }

    // load application's binary image file inside the area which start with 0x80400000 ( we put all application here and clear then went we change )
    #[deprecated]
    unsafe fn load_app(&self, app_id: usize) {
        // panic!("unused");
        if app_id >= self.num_app {
            panic!("All applications completed!");
        }
        info!("[kernel] Loading app_{}", app_id);
        // clear icache
        // we need to clear instruction-cache 
        // to allow running next app properly.  
        core::arch::asm!("fence.i");            // 调用了一个汇编的东西，实现了清除缓存
        // clear app area, set data from APP_BASE_ADDRESS.. to 0
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);   // BC batch we using a same location to load application so this address will be same.
        // copy [app_start..app_end] to [APP_BASE_ADDRESS..] so as to run it
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        debug!("finish APP_MANAGER.load_app");
        app_dst.copy_from_slice(app_src);
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}
// lazy_statc!{}: dynamicly generate global static variable
// lazy means only the first met initiate the variable.
// init static APP_MANAGER
lazy_static! {
    // UPSafeCell: prevent multi borrows
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                // static mut _num_app: u64;
                // fn _num_app();
                static apps: basic::AppMeta;
            }

            // get _num_app
            // let num_app_ptr = _num_app as usize as *const usize;
            // // read number of "should run" apps 
            // let num_app = num_app_ptr.read_volatile();
            // // array stores the start address of app
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            // // read address of app from _num_app, should from [1..?]
            // let app_start_raw: &[usize] =
            // core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            // // app_start[0..num_app] stores app start addresses
            // app_start[..=num_app].copy_from_slice(app_start_raw);
            // // store infos into AppManager
            AppManager {
                num_app: apps.len() as usize,
                current_app: 0,
                app_start,
            }
        })
    };
}
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// 1. load instruction
/// 2. push TrapContext into kernelStack
pub fn run_next_app() -> ! {
    extern "C" {
        static apps: basic::AppMeta;
    }
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    if current_app == app_manager.num_app {
        // debug!("inside");
        panic!("all application has been run successed");
    }
    debug!("you are now running application: {}", current_app);
    unsafe {
        apps.load(current_app);
    }
    // copy app instruction to APP_BASR_ADDRESS
    app_manager.move_to_next_app();                 // add account
    drop(app_manager);
    
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        // push TrapContext into kernelStack
        __restore(KERNEL_STACK[current_app].push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK[current_app].get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}
