#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{map, tracepoint},
    maps::HashMap,
    programs::TracePointContext,
};
use ebpf_monitor_common::SomaticMetrics;

#[map]
static METRICS: HashMap<u32, SomaticMetrics> = HashMap::with_max_entries(1, 0);

#[tracepoint(category = "syscalls", name = "sys_enter_execve")]
pub fn handle_execve(_ctx: TracePointContext) -> u32 {
    update_metrics(0); // 0 for execve
    0
}

#[tracepoint(category = "syscalls", name = "sys_enter_openat")]
pub fn handle_openat(_ctx: TracePointContext) -> u32 {
    update_metrics(1); // 1 for openat
    0
}

#[tracepoint(category = "syscalls", name = "sys_enter_connect")]
pub fn handle_connect(_ctx: TracePointContext) -> u32 {
    update_metrics(2); // 2 for connect
    0
}

#[tracepoint(category = "syscalls", name = "sys_enter_accept")]
pub fn handle_accept(_ctx: TracePointContext) -> u32 {
    update_metrics(2); // 2 for accept
    0
}

#[tracepoint(category = "syscalls", name = "sys_enter_read")]
pub fn handle_read(_ctx: TracePointContext) -> u32 {
    update_metrics(3); // 3 for read
    0
}

#[tracepoint(category = "syscalls", name = "sys_enter_write")]
pub fn handle_write(_ctx: TracePointContext) -> u32 {
    update_metrics(3); // 3 for write
    0
}

#[tracepoint(category = "syscalls", name = "sys_enter_clone")]
pub fn handle_clone(_ctx: TracePointContext) -> u32 {
    update_metrics(4); // 4 for clone (process pressure)
    0
}

#[inline(always)]
fn update_metrics(id: u32) {
    unsafe {
        if let Some(metrics) = METRICS.get_ptr_mut(&0) {
            match id {
                0 => (*metrics).execve_count += 1,
                1 => (*metrics).openat_count += 1,
                2 => (*metrics).tcp_connections += 1,
                3 => (*metrics).io_bursts += 1,
                4 => (*metrics).process_pressure += 1,
                _ => {}
            }
        } else {
            let mut initial = SomaticMetrics {
                execve_count: 0,
                openat_count: 0,
                tcp_connections: 0,
                io_bursts: 0,
                process_pressure: 0,
                last_timestamp: 0,
                // sovereign_word e pressure_level são zero-init aqui.
                // O userspace (OmniMind) os preenche via map update a cada ciclo.
                sovereign_word: [0u8; 32],
                pressure_level: 0,
            };
            match id {
                0 => initial.execve_count = 1,
                1 => initial.openat_count = 1,
                2 => initial.tcp_connections = 1,
                3 => initial.io_bursts = 1,
                4 => initial.process_pressure = 1,
                _ => {}
            }
            let _ = METRICS.insert(&0, &initial, 0);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
