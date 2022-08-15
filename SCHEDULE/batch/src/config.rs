/// provide the base appliances of batch
/// 根据要求，我们需要先将app全部装载到内存中，然后再通过load_app的方法，实现将app.image中的数据逐步转移到0x80400000的位置运行对应的程序
/// 本文件中定义如下内容：
/// 2. UserStack and KernelStack
/// 3. UPSafeCell
#[repr(align(4096))]
pub struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
pub struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

pub static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
pub static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}



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