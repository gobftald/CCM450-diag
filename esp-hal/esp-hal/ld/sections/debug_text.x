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

        *(.text.*esp_hal*soc*)

        *(.text.*esp_hal*sync*)


    } > ROTEXT

.esp-embassy : ALIGN(4)
    {
        *(.text.*esp_hal_embassy8Executor*)
        *(.text.*esp_hal_embassy11time_driver*)
        *(.text.*esp_hal_embassy11timer_queue*)
        *(.text.*esp_hal_embassy*timers*)

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

    } > ROTEXT

.esp-alloc : ALIGN(4)
    {
        *(.text.*esp_alloc*EspHeap*)
        *(.text.*linked_list_allocator*)
        *(.text.*allocator_api*)
    } > ROTEXT

INCLUDE "debug_esp_wifi.x"

INCLUDE "debug_libcore.x"

INCLUDE "debug_libprintf.x"

INCLUDE "debug_libpp.x"

INCLUDE "debug_libphy.x"

INCLUDE "debug_libnet80211.x"

INCLUDE "debug_libwpa-suppl.x"

.core : ALIGN(4)
    {
        *(.text.*alloc*alloc*)
        *(.text.*alloc*)
        *(.text.*core*alloc*)
        *(.text.*core*cell*)
        *(.text.*core*slice*)

        *(.text.memset)
        *(.text.memcpy)
        *(.text.memcmp)
        *(.text.memmove)

        *(.text.strlen)

        *(.text.*compiler_builtins*)

        *(.text.*Lanon*c503613613ad7ebc*)   /* __ledf2, __eqdf2 */

        *(.text.*__adddf3)
        *(.text.*__bswapsi2)
        *(.text.*__ctzsi2)
        *(.text.*__divdf3)
        *(.text.*__divdi3)
        *(.text.*__divsf3)
        *(.text.*__extendsfdf2)
        *(.text.*__fixdfsi)
        *(.text.*__fixunsdfsi)
        *(.text.*__floatsidf)
        *(.text.*__floatundisf)
        *(.text.*__floatunsidf)
        *(.text.*__gedf2)
        *(.text.*__gtdf2)
        *(.text.*__ltdf2)
        *(.text.*__moddi3)
        *(.text.*__muldf3)
        *(.text.*__nedf2)
        *(.text.*__subdf3)
        *(.text.*__udivdi3)
        *(.text.*__umoddi3)

    } > ROTEXT
