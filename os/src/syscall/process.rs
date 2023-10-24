//! Process management syscalls
use crate::{
    config::{CLOCK_FREQ, MAX_SYSCALL_NUM, PAGE_SIZE},
    mm::{translated_mut_type, MapPermission, PageTable, StepByOne, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task,
        suspend_current_and_run_next, TaskStatus,
    },
    timer::{get_time, get_time_us, MSEC_PER_SEC},
};

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

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let ts: &mut TimeVal = translated_mut_type(current_user_token(), ts as usize);
    ts.sec = us / 1_000_000;
    ts.usec = us % 1_000_000;
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let ti: &mut TaskInfo = translated_mut_type(current_user_token(), ti as usize);
    let task = get_current_task();
    ti.status = TaskStatus::Running;
    let current = get_time();
    ti.time = (current - task.program_start_time) / (CLOCK_FREQ / MSEC_PER_SEC);
    ti.syscall_times
        .as_mut_slice()
        .copy_from_slice(task.program_syscall_times.as_slice());
    0
}

// YOUR JOB: Implement mmap.
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

    let mut permission = MapPermission::empty();
    permission.set(MapPermission::R, port & 0x1 != 0);
    permission.set(MapPermission::W, port & 0x2 != 0);
    permission.set(MapPermission::X, port & 0x4 != 0);
    permission.set(MapPermission::U, true);

    let need_pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut start_vpn = start_va.floor();
    let mut end_va = VirtAddr::from(start + len - 1);
    let page_table = PageTable::from_token(current_user_token());
    let task = get_current_task();
    for _ in 0..need_pages {
        // 虚拟地址是否已经映射
        let pte = page_table.translate(start_vpn);
        if let Some(pte) = pte {
            // 虚拟地址已存在映射
            if pte.is_valid() {
                return -1;
            }
        }
        start_vpn.step();
    }
    // 跨页
    if end_va.page_offset() == 0 {
        end_va = VirtAddr::from(end_va.0 + 1);
    }
    task.memory_set
        .insert_framed_area(start_va, end_va, permission);
    0
}

// YOUR JOB: Implement munmap.
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
    let task = get_current_task();
    task.memory_set.unmap(start_va, end_va)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
