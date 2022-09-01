/// provide the base appliances of batch
/// 根据要求，我们需要先将app全部装载到内存中，然后再通过load_app的方法，实现将app.image中的数据逐步转移到0x80400000的位置运行对应的程序
/// 本文件中定义如下内容：
/// 2. UserStack and KernelStack
/// 3. UPSafeCell

use basic::*;

#[repr(align(4096))]
#[derive(Clone, Copy)]
pub struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
pub struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

pub static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

pub static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];
// pub static KERNEL_STACK: KernelStack = KernelStack {
//     data: [0; KERNEL_STACK_SIZE],
// };
// pub static USER_STACK: UserStack = UserStack {
//     data: [0; USER_STACK_SIZE],
// };

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> usize {
        // let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        // unsafe {
        //     *cx_ptr = cx;
        // }
        // unsafe { cx_ptr.as_mut().unwrap() }
        let task_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *task_cx_ptr = cx;
        }
        task_cx_ptr as usize
    }
}

impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(
        TrapContext::app_init_context(get_base_i(app_id), USER_STACK[app_id].get_sp())
    )
}

extern "C" {
    /// switch to context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}