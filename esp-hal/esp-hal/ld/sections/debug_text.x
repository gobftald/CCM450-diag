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
        *(.text.*esp_hal*interrupt*riscv*vectored*init_vectoring*)
        *(.text.hal_main)
        *(.text.*esp_hal4init*)
        *(.text.unlikely.EspDefaultHandler)
        *(.text._critical_section_1_0_acquire)
        *(.text._critical_section_1_0_release)

    } > ROTEXT

.esphal-clock : ALIGN(4)
    {
        *(.text.*esp_hal*clock*)
    } > ROTEXT

.esphal-timer : ALIGN(4)
    {
        *(.text.*esp_hal5timer*)
    } > ROTEXT

.esphal-embsy : ALIGN(4)
    {
        *(.text.*esp_hal_embassy8Executor*)
        *(.text.*esp_hal_embassy11time_driver*)
        *(.text.*esp_hal_embassy11timer_queue*)
        *(.text.*esp_hal_embassy*timers*)
    } > ROTEXT

.embassy-exec : ALIGN(4)
    {
        *(.text.*embassy_executor*)
        *(.text.__pender)
    } > ROTEXT

.embassy-time : ALIGN(4)
    {
        *(.text.*embassy_time*)
        *(.text.*_embassy_time*)
    } > ROTEXT

.esphal-alloc : ALIGN(4)
    {
        *(.text.*esp_alloc*)
        *(.text.*linked_list_allocator*)
    } > ROTEXT
