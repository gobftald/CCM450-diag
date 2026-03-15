.esp-hal : ALIGN(4)
{
    /* riscv_rt crate (dont't make a standalone script onnly for one function) */
    *(.text._default_setup_interrupts)

    /* esp-hal continue/start */
    
    *(.text._setup_interrupts)
    *(.text.unlikely.ExceptionHandler)
    *(.text.unlikely.EspDefaultHandler) 


    *(.text.*esp_hal*rom*regi2c*RawRegI2cField*write_field*)


    *(.text.*esp_hal*system*PeripheralGuard*new*)
    *(.text.*esp_hal*system*GenericPeripheralGuard*new*)
    *(.text.*esp_hal*system*PeripheralClockControl*enable_forced_with_counts*)
    *(.text.*esp_hal*system*assert_peri_reset_racey*)


    *(.text.*esp_hal*debugger*watchpoint_hit*)
    *(.text.*esp_hal*debugger*set_stack_watchpoint*)


    *(.text.*esp_hal*rng*fill_ptr_range*)


    *(.text.*esp_hal*peripherals*UART0*as*esp_hal*uart*Instance*parts*__esp_hal_internal_irq_handler*)
    *(.text.*esp_hal*peripherals*UART1*as*esp_hal*uart*Instance*parts*__esp_hal_internal_irq_handler*)


    *(.text.*esp_hal*interrupt*InterruptStatusIterator*as*core*iter*traits*iterator*Iterator*next*)
    *(.text.*esp_hal*interrupt*riscv*classic*current_runlevel*)
    *(.text.*esp_hal*interrupt*riscv*vectored*bind_interrupt*)


    *(.text.*esp_hal*gpio*PinGuard*as*core*ops*drop*Drop*drop*)


    *(.text.*core*ptr*drop_in_place*esp_hal*uart*UartRx*esp_hal*Async*read_async*closure*)

    *(.text.*esp_hal*uart*UartRx*esp_hal*Async*read_async*losure*)
    *(.text.*esp_hal*uart*UartTx*Dm*write*)

    *(.text.*esp_hal*uart*AnyUart*set_interrupt_handler*)

    *(.text.*esp_hal*uart*intr_handler*)
    *(.text.*esp_hal*uart*rx_event_check_for_error*)

    *(.text.*esp_hal*uart*Info*rxfifo_reset*)
    *(.text.*esp_hal*uart*Info*txfifo_reset*)
    *(.text.*esp_hal*uart*Info*enable_listen*)
    *(.text.*esp_hal*uart*Info*clear_rx_events*)
    *(.text.*esp_hal*uart*Info*clear_interrupts*)
    *(.text.*esp_hal*uart*Info*rx_events*)

    *(.text.*core*ptr*drop_in_place*esp_hal*uart*UartRx*esp_hal*Async*wait_for_buffered_data*closure*)


    *(.text.*esp_hal*fmt*__unwrap_failed*)


    /* esp-println crate */
    *(.text.*esp_println*Printer*write_bytes*)
    *(.text.*esp_println*Printer*as*core*fmt*.Write*write_str*)

} > ROTEXT
