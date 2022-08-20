use basic::*;
use output::*;
use output::log::*;
use loader::*;

use crate::__switch;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

#[derive(Copy, Clone)]
struct TaskControlBlock {
    task_cx: TaskContext,
    task_status: TaskStatus,
}

use lazy_static::*;
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock {
                task_cx: TaskContext::zero_init(),
                task_status: TaskStatus::UnInit,
            }; 
            MAX_APP_NUM 
        ];
        // init_app_cx  ：初始化任务上下文
        // goto_restore ：构造每个人物保存在任务控制块中的任务上下文  
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe { UPSafeCell::new(TaskManagerInner{
                tasks,
                current_task: 0,
            })},
        }
    };
}

// use super::__switch;
// use basic::UPSafeCell;
// impl TaskManager {
//     /// 找到一个就绪状态的程序，返回他的编号
//     fn find_next_task(&self) -> Option<usize> {
//         let inner = self.inner.exclusive_access();
//         let current = inner.current_task;
//         (current + 1..current + self.num_app + 1)
//             .map(|id| id % self.num_app)
//             .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
//     }

//     fn run_first_task(&self) -> !{
//         let mut inner = self.inner.exclusive_access();
//         let task0 = &mut inner.tasks[0];

//         task0.task_status = TaskStatus::Running;
//         let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
//         drop(inner);
//         let mut _unused = TaskContext::zero_init();
//         unsafe {
//             __switch(_unused, next_task_cx_ptr);
//         }
//         unreachable!();
//     }   

//     fn run_next_task(&self) {
//         if let Some(next) = self.find_next_task() {
//             let mut inner = self.inner.exclusive_access();
//             let current = inner.current_task;
//             // change status
//             inner.tasks[next].task_status = TaskStatus::Running;
//             inner.current_task = next;

//             let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
//             let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
//             drop(inner); // if you never drop here then it will error when you borrow it next time
//             unsafe {
//                 __switch(current_task_cx_ptr, next_task_cx_ptr);
//             }         
//             unreachable!();
//         }
//         else {
//             panic!("All applications hae been completed!");
//         }
//     }

//     pub fn suspend_current_task() {
//         let mut inner = self.inner.exclusive_access();
//         let current = inner.current_task;
//         inner.tasks[current].task_statux = TaskStatus::Ready;
//         drop(inner);
//     }


// }
impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    pub fn run_first_task(&self) -> ! {
        debug!("start run_first_taks");
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];

        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        debug!("using __switch");
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    pub fn suspend_current_task(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    pub fn exit_current_task(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    pub fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            // self.record_current_first_run_time(next);
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;

            // let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            // if let Some(current_task_cx_ptr) = &mut inner.tasks[current].task_cx as *mut TaskContext {
            //     drop(inner); 
            //     unsafe {
            //         __switch(current_task_cx_ptr, next_task_cx_ptr);
            //     }
            // } else {
            //     drop(inner);
            //     let mut __unused = TaskContext::zero_init();
            //     unsafe {
            //         __switch(&mut __unused as *mut TaskContext, next_task_cx_ptr);
            //     }
            // };
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    // TODO LAB1: Try to implement your function to update or get task info!

}