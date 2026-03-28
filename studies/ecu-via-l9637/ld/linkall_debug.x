/* copy of the original linkall.x starts here */
INCLUDE "memory.x"

REGION_ALIAS("ROTEXT", IROM);
REGION_ALIAS("RODATA", DROM);

REGION_ALIAS("RWDATA", DRAM);
REGION_ALIAS("RWTEXT", IRAM);

REGION_ALIAS("RTC_FAST_RWTEXT", RTC_FAST);
REGION_ALIAS("RTC_FAST_RWDATA", RTC_FAST);

/* INCLUDE "esp32c3.x"  // pasting the copy of esp32c3.x here
                        // to inject our scripts to the proper places */

/* esp32c3 fixups */

SECTIONS {
  .rotext_dummy (NOLOAD) :
  {
    /* This dummy section represents the .rodata section within ROTEXT.
    * Since the same physical memory is mapped to both DROM and IROM,
    * we need to make sure the .rodata and .text sections don't overlap.
    * We skip the amount of memory taken by .rodata* in .text
    */

    /* Start at the same alignment constraint than .flash.text */

    . = ALIGN(ALIGNOF(.rodata));
    . = ALIGN(ALIGNOF(.rodata.wifi));

    /* Create an empty gap as big as .text section */

    . = . + SIZEOF(.rodata_desc);
    . = . + SIZEOF(.rodata);
    . = . + SIZEOF(.rodata.wifi);

    /* Prepare the alignment of the section above. Few bytes (0x20) must be
     * added for the mapping header.
     */

    . = ALIGN(0x10000) + 0x20;
    _rotext_reserved_start = .;
  } > ROTEXT
}
/*INSERT BEFORE .text;*/
INSERT BEFORE .init;      /* I rewrite the first RODATA segment from .text to .init
                             see later in debug_init.x */

/* Similar to .rotext_dummy this represents .rwtext but in .data */
SECTIONS {
  .rwdata_dummy (NOLOAD) : ALIGN(4)
  {
    . = . + SIZEOF(.rwtext) + SIZEOF(.rwtext.wifi) + SIZEOF(.trap);
  } > RWDATA
}
INSERT BEFORE .data;

/* end of esp32c3 fixups */

/* Shared sections - ordering matters */
SECTIONS {
  INCLUDE "rwtext.x"
  INCLUDE "rwdata.x"
}

INCLUDE "rodata.x"

/* INCLUDE "text.x"     /* the modified and splitted copy of text.x comes later */

/* my insertions instead of the original text.x */
SECTIONS {
    INCLUDE "debug_init.x"        /* including the modified copy of text.x here
                                     to inject our scripts to the proper places 
                                     I split text.x into two parts
                                     .init stuff is here, in the first part */

    INCLUDE "debug_enumset.x"
    INCLUDE "debug_esp_rom_sys.x"
    INCLUDE "debug_esp_sync.x"
    INCLUDE "debug_esp_hal.x"

    INCLUDE "debug_linked_list_allocator.x"
    INCLUDE "debug_esp_alloc.x"
    INCLUDE "debug_heapless.x"
    INCLUDE "debug_embassy.x"
    INCLUDE "debug_esp_rtos.x"
    /*INCLUDE "debug_esp_wifi_sys.x"*/
    INCLUDE "debug_esp_radio.x"

    INCLUDE "debug_libcore.x"
    INCLUDE "debug_libprintf.x"
    INCLUDE "debug_libpp.x"
    INCLUDE "debug_libphy.x"
    INCLUDE "debug_libnet80211.x"
    INCLUDE "debug_libwpa-suppl.x"

    INCLUDE "debug_smoltcp.x"
    INCLUDE "debug_core.x"
    INCLUDE "debug_app.x"

    INCLUDE "debug_text_etc.x"      /* including the modified copy of text.x here
                                     to inject our scripts to the proper places
                                     I split text.x into two parts
                                     .init stuff is here, in the second part */
}
/* eof my insertions instead of the original text.x */

INCLUDE "rtc_fast.x"
INCLUDE "stack.x"
INCLUDE "dram2.x"
INCLUDE "metadata.x"
INCLUDE "eh_frame.x"
/* End of Shared sections */

_dram_data_start = ORIGIN( DRAM ) + SIZEOF(.trap) + SIZEOF(.rwtext);

/* end of pasting the copy of esp32c3.x */

INCLUDE "hal-defaults.x"
/* end of the copy of the original linkall.x */
