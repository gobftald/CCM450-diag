.esp-riscv-rt : ALIGN(4)
    {
        *(.init)
        *(.init.rust)
        *(.text.abort)
        *(.text.DefaultExceptionHandler)
        *(.text.default_post_init)

    } > ROTEXT

/*
.console : ALIGN(4)
    {
        *(.text.*console*print*)
        *(.text.*console*Printer*) 

    } > ROTEXT
*/

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

        *(.text.*esp_hal*uart*UartBuilder*init*)
        *(.text.*esp_hal*uart*UartTx*write*)
        *(.text.*esp_hal*uart*UartRx*apply_config*)
        *(.text.*esp_hal*uart*Uart*apply_config*)
        *(.text.*esp_hal*uart*Uart*set_at_cmd*)
        *(.text.*esp_hal*uart*Uart*with_tx*)
        *(.text.*esp_hal*uart*rx_event_check_for_error*)
        *(.text.*esp_hal*uart*Info*set_rx_fifo_full_threshold*)
        *(.text.*esp_hal*uart*Info*rx_fifo_full_threshold*)
        *(.text.*esp_hal*uart*Info*rxfifo_reset*)
        *(.text.*esp_hal*uart*Info*tx_fifo_count*)
        *(.text.*esp_hal*uart*Info*rx_fifo_count*)
        *(.text.*esp_hal*uart*Info*read_buffered*)
        *(.text.*esp_hal*uart*UartRxFuture*)
        *(.text.*esp_hal*uart*UartRx*)

        *(.text.*esp_hal*gpio*GpioBank*write_out_en*)
        *(.text.*esp_hal*gpio*GpioBank*write_output*)
        *(.text.*esp_hal*gpio*Flex*apply_output_config*)
        *(.text.*esp_hal*gpio*is_int_enabled*)
        *(.text.*esp_hal*gpio*PinGuard*as*core*ops*drop*Drop*drop*)
        *(.text.*esp_hal*gpio*interconnect*InputSignal*as*PeripheralSignal*connect_input_to_peripheral*)
        *(.text.*esp_hal*gpio*interconnect*OutputSignal*as*PeripheralSignal*connect_input_to_peripheral*)
        *(.text.*esp_hal*gpio*interconnect*OutputSignal*connect_with_guard*)

        *(.text.*esp_hal*rtc_cntl*)

        *(.text.*esp_hal*soc*implementation*)

        *(.text.*esp_hal*sync*)

        *(.text.*esp_hal*system*PeripheralGuard*as*core*ops*drop*Drop*drop*)

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
        *(.text.*embassy_net*udp*UdpSocket*poll_recv_from*)
        *(.text.*embassy_net*Stack*is_link_up*)
        *(.text.*embassy_net*Stack*config_v4*)
        *(.text.*embassy_net*Stack*with_mut*)

        *(.text.*embassy_future*select*Select*)

        *(.text.*core*future*poll_fn*PollFn*as*core*future*future*Future*poll*)
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

        *(.text.*core*slice*index*slice_index_fail*)
        *(.text.*core*slice*index*slice_index_fail*do_panic*runtime*)

        /* it takes 544 bytes, so we use only in udp test server */
        *(.text.*core*str*converts*from_utf8*)
        *(.text.*core*slice*impl*copy_from_slice*len_mismatch_fail*do_panic*runtime*)

    } > ROTEXT
