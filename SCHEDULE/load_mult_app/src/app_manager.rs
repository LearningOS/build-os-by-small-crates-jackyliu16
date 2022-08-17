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


/// trait AppManager { [lazy_static]
///     pub fn print_app_info(&self),                   // print the load information
///         #[try to deprecated]: 感觉可以直接通过loader::load_app实现对应内容，没有特别的必要
///         fn load_app(&self, app_id: usize),          // load application [we switch to it]
///         将load_app改名成为switch_app?   // BC if we switch in app_manager unable to release the exclusive_access automatically! thus we could only finish it outside 
///         fn switch_app(&self, app_id: usize),        // switch to a app
///     pub fn get_current_app(&self) -> usize,         // set current app number 
///     pub fn set_current_app(&self, app_id: usize)    // set current app num
///     #[deprecated]: 认为在所有我们需要切换应用的情况下，我们都首先知道了当前应用号，因此这个操作没有特别大的意义，不如扩大操作以实现更高的灵活性
///     pub fn move_to_next_app(&self, app_id: usize)
///     
/// }
/// pub fn print_app_info() 
/// pub fn run_next_app() -> !
///     1. load instruction
///     2. push TrapContext into kernelStack



impl AppManager {
    // prints total app number, and their start and end addresses
    // TODO: 
    pub fn print_app_info(&self) {
        info!("[kernel] num_app = {}", self.num_app);
        info!("All application has been linked at this location( in memory )");
        for i in 0..self.num_app {
            info!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
        debug!("KERNEL_STACK:{:X}", KERNEL_STACK.get_sp());
        debug!("USER_STACK:{:X}", USER_STACK.get_sp());
        // BC the different between kernel stack and user stack is 0x1000, which indicate we alloc correct.
    }

    // load application's binary image file inside the area which start with 0x80400000 ( we put all application here and clear then went we change )
    /// TODO: 修改此方法为，读取并切换到某个application
    // logical:
    // 1. if app_start wasn't >= 0x80400000 => hasn't been load => load application [switch]  
    // 2. else                              => has been load => [switch]
    // BC load_mult_app system actually haven't the need to change between different application so we treat it as change location of applications
    pub fn load_app(&self, app_id: usize) {
        // if app_id >= self.num_app {
        //     panic!("All applications completed!");
        // }
        // info!("[kernel] Loading app_{}", app_id);
        
        // // switch to another application
        // debug!("start switch to application");
        // debug!("APP_ADDRESS: {:X} to {:X}", self.app_start[app_id], self.app_start[app_id] + APP_SIZE_LIMIT);

        // extern "C" {
        //     fn __restore(cx_addr: usize);
        // }
        // unsafe {
        //     __restore(KERNEL_STACK.push_context(
        //         TrapContext::app_init_context(self.app_start[app_id], USER_STACK.get_sp())
        //     ) as *const _ as usize);
        // }
        unreachable!();
    }

    // unable to finish: if we switch here will case a non-returnable loan
    // fn switch_app(&self, app_id: usize) {
    // 
    // }
    
    pub fn set_current_app(&mut self, app_id: usize) {
        if app_id >= MAX_APP_NUM {
            panic!("try to load a application number which surpass limit");
        }
        self.current_app = app_id;
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }
    
    #[deprecated]
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
            
            // before this line we have successed load all application in the location of app_start
            // and all application has been load in the address start from 0x80218790?
            // so we just load it in the address from 0x8400000
            // core::arch::asm!("fence.i");

            // load application in memory
            for i in 0..num_app {
                let base_i = APP_BASE_ADDRESS + i * APP_SIZE_LIMIT;
                // the app_start[i] here is stand for the location where they has been load in memeory now
                loader::load_app(app_start[i], app_start[i+1], base_i);
                // update app_start
                app_start[i] = base_i;
            }

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
use crate::config::USER_STACK;
/// 1. load instruction
/// 2. push TrapContext into kernelStack
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    
    let current_app = app_manager.get_current_app();
    if current_app == app_manager.num_app {
        // debug!("inside");
        panic!("all application has been run successed");
    }
    // copy app instruction to APP_BASR_ADDRESS
    let next_app_num = current_app + 1;
    app_manager.set_current_app(next_app_num);                 // add account
    let next_ptr = app_manager.app_start[next_app_num];
    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources
    debug!("you are now loading application:{}",next_app_num);
    debug!("restore to address in :{:x}", next_ptr);
    debug!("USER_STACK.get_sp:{:X}", USER_STACK.get_sp());
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        // push TrapContext into kernelStack
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS + current_app * APP_SIZE_LIMIT,
            USER_STACK.get_sp(),
        )) as *const _ as usize)
    }
    panic!("Unreachable in batch::run_current_app!");
}
