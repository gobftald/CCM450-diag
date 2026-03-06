//! Since current rtos-trace crate (its C functions) is targetting only Cortex-M, I need a minor 
//! customization of systemview_target's build.rs (only commenting out SEGGER_RTT_ASM_ARMv7M.S)
//! but replacinng its configuration via heavy unsafe external hacks in the beginning of main.


// Satisfy the missing architecture-specific symbols for RISC-V.
// without Systemview-target's "callbacks-os-time" feature we deafulted to these function name
// in SEGGER_SYSVIEW_ConfDefaults.h and these are expected to be provided manually.
#[unsafe(no_mangle)]
pub extern "C" fn SEGGER_SYSVIEW_X_GetTimestamp() -> u32 {
    // Get the internal RISC-V cycle count (4ns precision at 160MHz)
    //esp_hal::cpu::get_cycle_count() // using this fucntion we read mpccr, but it is disbaled during wifi mode
    
    // hard coded direct access to systimer 0 !!!
    unsafe { (*esp_hal::peripherals::SYSTIMER::PTR).unit_value(0).lo().read().bits() }
}

#[unsafe(no_mangle)]
pub extern "C" fn SEGGER_SYSVIEW_X_GetInterruptId() -> u32 {
    // Return 0 or the current interrupt status to satisfy the tracer
    0 
}

// Define the OS API structure
// The fields must match the SEGGER C-struct layout exactly.
#[repr(C)]
pub struct SEGGER_SYSVIEW_OS_API {
    // C: U64 (*pfGetTime)(void)
    pub pf_get_time: unsafe extern "C" fn() -> u64,

    // C: void (*pfSendTaskList)(void)
    pub pf_send_task_list: unsafe extern "C" fn(),
}

type SendSysDescFn = unsafe extern "C" fn();

unsafe extern "C" {
    pub fn SEGGER_SYSVIEW_Init(
        sys_freq: u32,
        cpu_freq: u32,
        p_os_api: *const SEGGER_SYSVIEW_OS_API,
        pf_send_sys_desc: Option<SendSysDescFn>,
    );
    fn SEGGER_SYSVIEW_SendSysDesc(desc: *const u8);
    fn SEGGER_SYSVIEW_SetRAMBase(addr: u32);
    //pub fn SEGGER_SYSVIEW_Start();
}

// We defined/implemented SEGGER_SYSVIEW_OS_API functions
// by rtos_trace::global_os_callbacks!(Scheduler); in esp-rtos.
// BTW _rtos_trace_time implemented exactly the same as SEGGER_SYSVIEW_X_GetTimestamp
// except returning with u64
unsafe extern "C" {
    fn _rtos_trace_time() -> u64;
    fn _rtos_trace_task_list();
}

static SYSVIEW_OS_API: SEGGER_SYSVIEW_OS_API = SEGGER_SYSVIEW_OS_API {
    pf_get_time: _rtos_trace_time,
    pf_send_task_list: _rtos_trace_task_list,
};

// Define SendSysDesc callback
pub unsafe extern "C" fn send_sys_desc_callback() {
    unsafe { SEGGER_SYSVIEW_SendSysDesc(
        b"N=ecu-via-elm327,C=riscv32,D=ESP32C3,M=RAM,0x3FC80000,0x80000,O=esp-rtos\0".as_ptr()); }
}

// config/init for ESP32C3
#[unsafe(no_mangle)]
pub extern "C" fn SEGGER_SYSVIEW_Conf() {
    unsafe {
        // Initialize using the Segger C logic
        SEGGER_SYSVIEW_Init(
            16_000_000,             // SYSTIMER
            160_000_000,            // 160MHz CPU
            &SYSVIEW_OS_API as *const _,
            Some(send_sys_desc_callback),
        );

    unsafe extern "C" {
        // from linker script, after .rwdata_dummy
        static _data_start: u8;
    }
    SEGGER_SYSVIEW_SetRAMBase(core::ptr::addr_of!(_data_start) as u32 );
    }
}
