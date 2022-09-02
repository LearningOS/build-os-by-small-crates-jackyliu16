use lazy_static::*;
use basic::*;
use crate::config::__switch;
use output::log::*;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,    // provide the status
    pub task_cx: TaskContext,       // saving the entry of application
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
use crate::config::init_app_cx;
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        extern "C" {
            static apps: basic::AppMeta;
        }
        let num_app = unsafe { apps.len() };
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
        task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];
        for i in 0..num_app {
            unsafe { apps.load(i); }
        }
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app: unsafe { apps.len() },
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks, 
                    current_task: 0 
                })
            }
        }
    };
}

impl TaskManager {

    /// 设想中我们通过提供一个新的东西，也就是内置一个基于TaskManager的分配器来实现我们的各种task的顺序获取
    /// pid: alloc but haven't dealloc
    /// 然后我们将对于第一个task的初始化部分放置在其中，就可以相当程度上解决了我们目前所面临的问题
    pub fn run_first_task(&self) -> ! {
        // debug!("inside run_first_task");
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        
        task0.task_status = TaskStatus::Running;
        let mut _unused = TaskContext::zero_init();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);

        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task");
    }

    /// 现在暂且使用原先这套操作，后面想看下能不能引入一些别的方案来实现模块化更替
    /// 就是感觉这两个部分内联太严重了，应该没有办法可以拆分出来
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
    pub fn run_next_task(&self) -> !{
        // debug!("inside run_next_task");
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            // self.record_current_first_run_time(next);
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;

            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
                drop(inner); 
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            panic!("unreachable in run_next_task");
        } else {
            panic!("All applications completed!");
        }
    }

    pub fn suspend_current_task(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;        
    }

    pub fn exited_current_task(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;        
    }

    // pub fn print_app_info(&self) {
        // debug!("inside print_app_info");
        // let inner = self.inner.exclusive_access();
        // for i in inner.tasks {
        //     debug!("=====");
        //     debug!("ra: {:?}, sp: {:?}, s: {:?}", i.task_cx.ra, i.task_cx.sp, i.task_cx.s);
        // }
    // }

}
