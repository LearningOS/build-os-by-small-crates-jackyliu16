/// 1. 在switch之前需要明知其需要切换到哪一个控制流，因此__switch由两个参数，一个是自己一个是需要切换到的控制流
/// 
#[derive(Copy, Clone, Debug)]
pub struct TaskContext {
    ra: usize,          // return address
    sp: usize,          // stack pointer
    s: [usize; 12],     // register that should be save by callee
}

impl TaskContext{
    pub fn zero_init() -> Self {
        Self { ra: 0, sp: 0, s: [0; 12] }
    }
    /// 实现在switch之后自动跳转到__restore的地址上的效果
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self { 
            ra: __restore as usize, 
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    UnInit,     // 未初始化
    Ready,      // 准备运行
    Running,    // 正在运行
    Exited,     // 已退出
}
