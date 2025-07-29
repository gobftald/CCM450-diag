.esp-riscv-rt : ALIGN(4)
    {
        *(.init)
        *(.init.rust)
        *(.text.abort)
        *(.text.DefaultExceptionHandler)
        *(.text.default_post_init)

    } > ROTEXT

.console : ALIGN(4)
    {
        *(.text.*console*print*)
        *(.text.*console*Printer*) 

    } > ROTEXT

.esp-hal : ALIGN(4)
    {
        *(.text._setup_interrupts)
        *(.text.*esp_hal*interrupt*riscv*vectored*enable*)
        *(.text.*esp_hal*interrupt*riscv*vectored*init_vectoring*)
        *(.text.hal_main)
        *(.text.*esp_hal4init*)
        *(.text.unlikely.EspDefaultHandler)
        *(.text._critical_section_1_0_acquire)
        *(.text._critical_section_1_0_release)

        *(.text.*esp_hal*clock*)

        *(.text.*esp_hal*timer*systimer*)
        *(.text.*esp_hal*timer*OneShotTimer*)
        *(.text.*esp_hal*timer*PeriodicTimer*)
        *(.text.*esp_hal*TIMG0*)

        *(.text.*esp_hal*rtc_cntl*)

        *(.text.*esp_hal*soc*)

        *(.text.*esp_hal*sync*)

    } > ROTEXT

.embassy : ALIGN(4)
    {
        *(.text.*embassy_executor*raw*poll_exited*)
        /* here are the two main tasks, (put into .main) */
        /* *(.text.*embassy_executor*raw*TaskStorage*) */
        *(.text.*embassy_executor*raw*util*)
        *(.text.*embassy_executor*raw*waker*)
        *(.text.*embassy_executor*raw*Executor*)
        *(.text.*embassy_executor*raw*wake_task*)
        *(.text.*embassy_executor*spawner*Spawner*)

        *(.text.__pender)

        *(.text.*embassy_time*)
        *(.text.*_embassy_time*)
        *(.text.*embassy_net*Inner*)
        *(.text.*embassy_net*new*)
        *(.text.*embassy_sync*)

    } > ROTEXT

.esp-embassy : ALIGN(4)
    {
        *(.text.*esp_hal_embassy8Executor*)
        *(.text.*esp_hal_embassy11time_driver*)
        *(.text.*esp_hal_embassy11timer_queue*)

    } > ROTEXT

.esp-alloc : ALIGN(4)
    {
        *(.text.*esp_alloc*EspHeap*)
        *(.text.*___rustc12___rust_alloc)
        *(.text.*___rustc14___rust_dealloc)
        *(.text.*___rustc14___rust_realloc)
        *(.text.*___rustc19___rust_alloc_zeroed)

        *(.text.*linked_list_allocator*)
        
        *(.text.*allocator_api*)

    } > ROTEXT

.port-atomic : ALIGN(4)
    {
        *(.text.*portable_atomic*AtomicBool*load*)
        *(.text.*portable_atomic*AtomicUsize*load*)
        *(.text.*portable_atomic*AtomicU32*load*)
        
        *(.text.*portable_atomic*AtomicUsize*store*)
        *(.text.*portable_atomic*AtomicU32*store*)

    } > ROTEXT

INCLUDE "debug_esp_wifi.x"

INCLUDE "debug_libcore.x"

INCLUDE "debug_libprintf.x"

INCLUDE "debug_libpp.x"

INCLUDE "debug_libphy.x"

INCLUDE "debug_libnet80211.x"

INCLUDE "debug_libwpa-suppl.x"

INCLUDE "debug_smoltcp.x"

.core : ALIGN(4)
    {
        *(.text.strlen)

        *(.text.*alloc*alloc*)
        *(.text.*alloc*raw_vec*)
        *(.text.*alloc*collections*)

        *(.text.*core*alloc*layout*Layout*is_size_align_valid*)

        *(.text.*core*cell*once*OnceCell*try_init*)
        *(.text.*core*cell*panic_already_borrowed*)
        *(.text.*core*cell*panic_already_mutably_borrowed*)

        *(.text.*core*slice*index*slice_start_index_len_fail*do_panic*runtime*)
        *(.text.*core*slice*index*slice_end_index_len_fail*do_panic*runtime*)
        *(.text.*core*slice*index*slice_index_order_fail*do_panic*runtime*)

    } > ROTEXT
