
use crate::config::*;
use lazy_static::*;
use basic::*;

struct AppManager {
    num_app: usize,
    current_app: usize,
}

impl AppManager {
    fn print_app_info(&self) {
        todo!();
    }
    // load application's binary image file inside the area which start with 0x80400000 ( we put all application here and clear then went we change )
    #[deprecated]
    unsafe fn load_app(&self, app_id: usize) -> ! {
        panic!("unreadable")
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            AppManager {
                num_app: unsafe { get_app_num() },
                current_app: 0,
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
    let current_app = app_manager.get_current_app();
    if current_app == app_manager.num_app {
        panic!("all application has been run successed");
    }

    unsafe { load_app(current_app, 0); }

    // copy app instruction to APP_BASR_ADDRESS
    app_manager.move_to_next_app();                 // add account
    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources
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
