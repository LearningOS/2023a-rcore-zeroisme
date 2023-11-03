use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::sync::UPSafeCell;

const MAX_THREADS: usize = 1000;
const MAX_RESOURCE: usize = 1000;

pub struct Bank {
    pub available: Vec<u32>,
    pub allocation: Vec<Vec<u32>>,
    pub need: Vec<Vec<u32>>,
}

impl Bank {
    pub fn new() -> Self {
        let mut v = Vec::new();
        for _ in 0..MAX_THREADS {
            v.push(vec![0; MAX_RESOURCE]);
        }
        let need = v.clone();
        Self {
            available: vec![0; MAX_RESOURCE],
            allocation: v,
            need,
        }
    }

    fn add_resource(&mut self, idx: usize, n: usize) {
        self.available[idx] = n as u32;
    }

    fn alloc(&mut self, ti: usize, ri: usize, n: usize) {
        self.available[ri] -= n as u32;
        self.allocation[ti][ri] += n as u32;
        self.need[ti][ri] -= n as u32;
    }

    fn dealloc(&mut self, ti: usize, ri: usize, n: usize) {
        self.available[ri] += n as u32;
        self.allocation[ti][ri] -= n as u32;
    }

    fn need(&mut self, ti: usize, ri: usize, n: usize) {
        self.need[ti][ri] += n as u32;
    }

    fn deneed(&mut self, tid: usize, rid: usize, n: usize) {
        self.need[tid][rid] -= n as u32;
    }
}

lazy_static! {
    pub static ref BANK: UPSafeCell<Bank> = unsafe { UPSafeCell::new(Bank::new()) };
    pub static ref ENABLE_DEADLOCK_DETECT: UPSafeCell<bool> = unsafe { UPSafeCell::new(false) };
}
/// check dead lock
pub fn check() -> bool {
    let enabled = ENABLE_DEADLOCK_DETECT.exclusive_access();
    if !*enabled {
        return true
    }
    let bank = BANK.exclusive_access();
    let mut work = bank.available.clone();
    let need = bank.need.clone();
    let alloc = bank.allocation.clone();
    let mut finish = vec![false; MAX_THREADS];

    let mut count = 0;

    while count < MAX_THREADS {
        let mut safe = false;

        for i in 0..MAX_THREADS {
            if !finish[i] {
                let mut can_running = true;
                for j in 0..MAX_RESOURCE {
                    if need[i][j] > work[j] {
                        can_running = false;
                        break;
                    }
                }

                if can_running {
                    finish[i] = true;
                    for j in 0..MAX_RESOURCE {
                        work[j] += alloc[i][j];
                    }
                    safe = true;
                    count += 1;
                }
            }
        }
        if !safe {
            return false;
        }
    }
    true
}

/// add resource
pub fn add_resource(rid: usize, n: usize) {
    BANK.exclusive_access().add_resource(rid, n)
}
/// alloc
pub fn bank_alloc(tid: usize, rid: usize, n: usize) {
    BANK.exclusive_access().alloc(tid, rid, n)
}
/// dealloc
pub fn bank_dealloc(tid: usize, rid: usize, n: usize) {
    BANK.exclusive_access().dealloc(tid, rid, n)
}
/// need
pub fn need(tid: usize, rid: usize, n: usize) {
    BANK.exclusive_access().need(tid, rid, n)
}

/// deneed
pub fn deneed(tid: usize, rid: usize, n: usize) {
    BANK.exclusive_access().deneed(tid, rid, n)
}

/// enable deadlock detect
pub fn enable_deadlock_detect() {
    *ENABLE_DEADLOCK_DETECT.exclusive_access() = true;
}