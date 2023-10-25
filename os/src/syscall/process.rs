//! Process management syscalls
//!

use crate::{
    config::{BIG_STRIDE, MAX_SYSCALL_NUM},
    fs::{open_file, OpenFlags},
    mm::{translated_refmut, translated_str, MapPermission, VirtAddr},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
    },
    timer::{get_time_ms, get_time_us},
};
use alloc::sync::Arc;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    //trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //trace!("kernel: sys_waitpid");
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let ts = translated_refmut(current_user_token(), ts);
    let us = get_time_us();
    ts.sec = us / 1_000_000;
    ts.usec = us % 1_000_000;
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ti = translated_refmut(current_user_token(), ti);
    let current_task = current_task().unwrap();
    let current_task_inner = current_task.inner_exclusive_access();
    ti.status = TaskStatus::Running;
    ti.time = get_time_ms() - current_task_inner.started_time;
    ti.syscall_times
        .as_mut_slice()
        .copy_from_slice(current_task_inner.syscall_times.as_slice());
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // port 其余位必须为0
    if port & !0x7 != 0 {
        return -1;
    }
    // 无意义分配
    if port & 0x7 == 0 {
        return -1;
    }
    let start_va = VirtAddr::from(start);
    // start 没有按页大小对齐
    if start_va.page_offset() != 0 {
        return -1;
    }
    let mut end_va = VirtAddr::from(start + len - 1);
    if end_va.page_offset() == 0 {
        end_va.0 += 1;
    }

    let mut permission = MapPermission::empty();
    permission.set(MapPermission::R, port & 0x1 != 0);
    permission.set(MapPermission::W, port & 0x2 != 0);
    permission.set(MapPermission::X, port & 0x4 != 0);
    permission.set(MapPermission::U, true);

    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.memory_set.mmap(start_va, end_va, permission)
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start_va = VirtAddr::from(start);
    if start_va.page_offset() != 0 {
        return -1;
    }
    let mut end_va = VirtAddr::from(start + len - 1);
    // 跨页
    if end_va.page_offset() == 0 {
        end_va = VirtAddr::from(end_va.0 + 1);
    }
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.memory_set.unmap(start_va, end_va)
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    let current_task = current_task().unwrap();
    trace!("kernel:pid[{}] sys_spawn[{}]", current_task.pid.0, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task.spawn(all_data.as_slice());
        add_task(task.clone());
        task.getpid() as isize
    } else {
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(prio: isize) -> isize {
    if prio < 2 {
        return -1;
    }
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.priority = prio as u8;
    task_inner.pass = BIG_STRIDE / task_inner.priority;
    prio
}
