#![allow(non_camel_case_types)]
#![no_std]
use core::marker::PhantomData;
use core::ops::Deref;

#[allow(unused_imports)]
// 11
use generic::*;
///cCommon register and bit access and modify traits
// 13
pub mod generic;

#[cfg(feature = "rt")]
// 15
extern "C" {
    fn WIFI_MAC();
    fn WIFI_MAC_NMI();
    fn WIFI_PWR();
    fn WIFI_BB();
    fn BT_MAC();
    fn BT_BB();
    fn BT_BB_NMI();
    fn RWBT();
    fn RWBLE();
    fn RWBT_NMI();
    fn RWBLE_NMI();
    fn I2C_MASTER();
    fn SLC0();
    fn SLC1();
    fn APB_CTRL();
    fn UHCI0();
    fn GPIO();
    fn GPIO_NMI();
    fn SPI1();
    fn SPI2();
    fn I2S0();
    fn UART0();
    fn UART1();
    fn LEDC();
    fn EFUSE();
    fn TWAI0();
    fn USB_DEVICE();
    fn RTC_CORE();
    fn RMT();
    fn I2C_EXT0();
    fn TIMER1();
    fn TIMER2();
    fn TG0_T0_LEVEL();
    fn TG0_WDT_LEVEL();
    fn TG1_T0_LEVEL();
    fn TG1_WDT_LEVEL();
    fn CACHE_IA();
    fn SYSTIMER_TARGET0();
    fn SYSTIMER_TARGET1();
    fn SYSTIMER_TARGET2();
    fn SPI_MEM_REJECT_CACHE();
    fn ICACHE_PRELOAD0();
    fn ICACHE_SYNC0();
    fn APB_ADC();
    fn DMA_CH0();
    fn DMA_CH1();
    fn DMA_CH2();
    fn RSA();
    fn AES();
    fn SHA();
    fn FROM_CPU_INTR0();
    fn FROM_CPU_INTR1();
    fn FROM_CPU_INTR2();
    fn FROM_CPU_INTR3();
    fn ASSIST_DEBUG();
    fn DMA_APBPERI_PMS();
    fn CORE0_IRAM0_PMS();
    fn CORE0_DRAM0_PMS();
    fn CORE0_PIF_PMS();
    fn CORE0_PIF_PMS_SIZE();
    fn BAK_PMS_VIOLATE();
    fn CACHE_CORE0_ACS();
}

#[doc(hidden)]
#[repr(C)]
// 81
pub union Vector {
    pub _handler: unsafe extern "C" fn(),
    pub _reserved: usize,
}

#[cfg(feature = "rt")]
#[link_section = ".rwtext"]
#[no_mangle]
// 89
pub static __EXTERNAL_INTERRUPTS: [Vector; 62] = [
    Vector { _handler: WIFI_MAC },
    Vector {
        _handler: WIFI_MAC_NMI,
    },
    Vector { _handler: WIFI_PWR },
    Vector { _handler: WIFI_BB },
    Vector { _handler: BT_MAC },
    Vector { _handler: BT_BB },
    Vector {
        _handler: BT_BB_NMI,
    },
    Vector { _handler: RWBT },
    Vector { _handler: RWBLE },
    Vector { _handler: RWBT_NMI },
    Vector {
        _handler: RWBLE_NMI,
    },
    Vector {
        _handler: I2C_MASTER,
    },
    Vector { _handler: SLC0 },
    Vector { _handler: SLC1 },
    Vector { _handler: APB_CTRL },
    Vector { _handler: UHCI0 },
    Vector { _handler: GPIO },
    Vector { _handler: GPIO_NMI },
    Vector { _handler: SPI1 },
    Vector { _handler: SPI2 },
    Vector { _handler: I2S0 },
    Vector { _handler: UART0 },
    Vector { _handler: UART1 },
    Vector { _handler: LEDC },
    Vector { _handler: EFUSE },
    Vector { _handler: TWAI0 },
    Vector {
        _handler: USB_DEVICE,
    },
    Vector { _handler: RTC_CORE },
    Vector { _handler: RMT },
    Vector { _handler: I2C_EXT0 },
    Vector { _handler: TIMER1 },
    Vector { _handler: TIMER2 },
    Vector {
        _handler: TG0_T0_LEVEL,
    },
    Vector {
        _handler: TG0_WDT_LEVEL,
    },
    Vector {
        _handler: TG1_T0_LEVEL,
    },
    Vector {
        _handler: TG1_WDT_LEVEL,
    },
    Vector { _handler: CACHE_IA },
    Vector {
        _handler: SYSTIMER_TARGET0,
    },
    Vector {
        _handler: SYSTIMER_TARGET1,
    },
    Vector {
        _handler: SYSTIMER_TARGET2,
    },
    Vector {
        _handler: SPI_MEM_REJECT_CACHE,
    },
    Vector {
        _handler: ICACHE_PRELOAD0,
    },
    Vector {
        _handler: ICACHE_SYNC0,
    },
    Vector { _handler: APB_ADC },
    Vector { _handler: DMA_CH0 },
    Vector { _handler: DMA_CH1 },
    Vector { _handler: DMA_CH2 },
    Vector { _handler: RSA },
    Vector { _handler: AES },
    Vector { _handler: SHA },
    Vector {
        _handler: FROM_CPU_INTR0,
    },
    Vector {
        _handler: FROM_CPU_INTR1,
    },
    Vector {
        _handler: FROM_CPU_INTR2,
    },
    Vector {
        _handler: FROM_CPU_INTR3,
    },
    Vector {
        _handler: ASSIST_DEBUG,
    },
    Vector {
        _handler: DMA_APBPERI_PMS,
    },
    Vector {
        _handler: CORE0_IRAM0_PMS,
    },
    Vector {
        _handler: CORE0_DRAM0_PMS,
    },
    Vector {
        _handler: CORE0_PIF_PMS,
    },
    Vector {
        _handler: CORE0_PIF_PMS_SIZE,
    },
    Vector {
        _handler: BAK_PMS_VIOLATE,
    },
    Vector {
        _handler: CACHE_CORE0_ACS,
    },
];

#[doc(hidden)]
// 208
pub mod interrupt;
pub use self::interrupt::Interrupt;

/// Interrupt Controller (Core 0)
// 855
pub struct INTERRUPT_CORE0 {
    _marker: PhantomData<*const ()>,
}

// 858
unsafe impl Send for INTERRUPT_CORE0 {}

// 859
impl INTERRUPT_CORE0 {
    /// Pointer to the register block
    pub const PTR: *const interrupt_core0::RegisterBlock = 0x600c_2000 as *const _;
}

// 886
impl Deref for INTERRUPT_CORE0 {
    type Target = interrupt_core0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

/// Interrupt Controller (Core 0)
// 898
pub mod interrupt_core0;
