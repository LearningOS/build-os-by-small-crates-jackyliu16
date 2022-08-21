use crate::config::*;
use lazy_static::*;
use output::log::*;
use basic::*;
use basic::UPSafeCell;

pub struct AppManager {
    pub num_app: usize,
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
    pub unsafe fn load_app(&mut self, app_id: usize) {
        use basic::{load_app, APP_BASE_ADDRESS, APP_SIZE_LIMIT};
        if app_id >= self.num_app {
            panic!("all application has been successed run");
        }
        info!("[KERNEL] Loading add_{}", app_id);

        let base_i = APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT;
        load_app(self.app_start[app_id], self.app_start[app_id + 1], base_i);

        self.app_start[app_id] = base_i;
        debug!("Loaded applicatio_{}", app_id);

        // switch to another application
        // extern "C" {
        //     fn __restore(cx_addr: usize);
        // }
        // unsafe {
        //     __restore(KERNEL_STACK.push_context(
        //         TrapContext::app_init_context(self.app_start[app_id], USER_STACK.get_sp())
        //     ) as *const _ as usize);
        // }
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
    pub static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                // static mut _num_app: u64;
                fn _num_app();
            }
            // get _num_app
            let num_app_ptr = _num_app as usize as *const usize;
            // read number of "should run" apps 
            let num_app = num_app_ptr.read_volatile();
            // array stores the start address of app
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            // read address of app from _num_app, should from [1..?]
            let app_start_raw: &[usize] =
            core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            // app_start[0..num_app] stores app start addresses
            app_start[..=num_app].copy_from_slice(app_start_raw);
            // store infos into AppManager
            AppManager {
                num_app,
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
    let mut app_manager = APP_MANAGER.exclusive_access(); 
    let current = app_manager.num_app;

    if current == app_manager.get_current_app() {
        panic!("All application has been run successed");
    }

    // BC we have load the application in right location so here we just change to another locatio 
    app_manager.move_to_next_app();
    drop(app_manager);

    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK[current].push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS + current * APP_SIZE_LIMIT, 
            USER_STACK[current].get_sp() 
        )) as *const _ as usize)
    }
    panic!("unreachable");


    // let mut app_manager = APP_MANAGER.exclusive_access();
    // let current_app = app_manager.get_current_app();
    // if current_app == app_manager.num_app {
    //     // debug!("inside");
    //     panic!("all application has been run successed");
    // }
    // unsafe {
    //     app_manager.load_app(current_app);
    // }
    // // copy app instruction to APP_BASR_ADDRESS
    // app_manager.move_to_next_app();                 // add account
    // drop(app_manager);
    // // before this we have to drop local variables related to resources manually
    // // and release the resources
    // debug!("APP_BASE_ADDRESS:{:x}", APP_BASE_ADDRESS);
    // debug!("USER_STACK.get_sp:{:X}", USER_STACK.get_sp());
    // extern "C" {
    //     fn __restore(cx_addr: usize);
    // }
    // unsafe {
    //     // push TrapContext into kernelStack
    //     __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
    //         APP_BASE_ADDRESS,
    //         USER_STACK.get_sp(),
    //     )) as *const _ as usize);
    // }
    // panic!("Unreachable in batch::run_current_app!");
}
