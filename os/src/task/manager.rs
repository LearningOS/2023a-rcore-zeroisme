//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

// pub struct StrideTaskManager {
//     ready_queue: BinaryHeap<Arc<TaskControlBlock>>,
// }

// impl StrideTaskManager {
//     pub fn new() -> Self {
//         Self {
//             ready_queue: BinaryHeap::new(),
//         }
//     }

//     /// Add process back to ready queue
//     pub fn add(&mut self, task: Arc<TaskControlBlock>) {
//         self.ready_queue.push(task);
//     }
//     /// Take a process out of the ready queue
//     pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
//         self.ready_queue.pop()
//     }
// }

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };

    // Stride Task Manager instance through lazy_static!
    // pub static ref TASK_MANAGER: UPSafeCell<StrideTaskManager> = 
    //     unsafe { UPSafeCell::new(StrideTaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
