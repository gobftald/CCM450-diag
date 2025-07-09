.esp-wifi : ALIGN(4)
    {
        *(.text.*esp_wifi*init_clocks*)

        *(.text.*esp_wifi*common_adapter*semphr_create*)
        *(.text.*esp_wifi*common_adapter*semphr_delete*)
        *(.text.*esp_wifi*common_adapter*read_mac*)
        *(.text.*esp_wifi*common_adapter*chip_specific*enable_wifi_power_domain*)
        *(.text.puts)
        *(.text.ets_timer_disarm)
        *(.text.ets_timer_setfn)

        /* compat */
        *(.text.strnlen)

        *(.text.*esp_wifi*preempt*enable*)
        *(.text.*esp_wifi*preempt*yield_task*)
        *(.text.*esp_wifi*preempt_builtin*task_switch*)
        *(.text.*esp_wifi*preempt_builtin*timer*timer_tick_handler*)
        *(.text.*esp_wifi*preempt_builtin*timer*arch_specific*setup_timer*)
        *(.text.FROM_CPU_INTR2) 
        *(.text.*esp_wifi*preempt_builtin*BuiltinScheduler*as*esp_wifi*preempt*Scheduler*task_create*)

        *(.text.*esp_wifi*tasks*init_tasks*)
        *(.text.*esp_wifi*tasks*timer_task*)

        *(.text.*esp_wifi*wifi*new*)
    
        *(.text.*esp_wifi*wifi*os_adapter*spin_lock_create*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_int_disable*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_int_restore*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_thread_semphr_get*)
        *(.text.*esp_wifi*wifi*os_adapter*recursive_mutex_create*)
        *(.text.*esp_wifi*wifi*os_adapter*mutex_lock*)
        *(.text.*esp_wifi*wifi*os_adapter*mutex_unlock*)
        *(.text.*esp_wifi*wifi*os_adapter*queue_send*)
        *(.text.*esp_wifi*wifi*os_adapter*queue_recv*)
        *(.text.*esp_wifi*wifi*os_adapter*task_create_pinned_to_core*)
        *(.text.*esp_wifi*wifi*os_adapter*task_delay*)
        *(.text.*esp_wifi*wifi*os_adapter*task_ms_to_tick*)
        *(.text.*esp_wifi*wifi*os_adapter*task_get_current_task*)
        *(.text.*esp_wifi*wifi*os_adapter*task_get_max_priority*)
        *(.text.*esp_wifi*wifi*os_adapter*log_timestamp*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_malloc*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_calloc*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_zalloc*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_create_queue*)
        *(.text.*esp_wifi*wifi*os_adapter*wifi_delete_queue*)
        *(.text.*esp_wifi*wifi*os_adapter*coex_schm_register_cb_wrapper*)
        *(.text.*esp_wifi*wifi*os_adapter*coex_register_start_cb*)

        *(.text.*esp_wifi*wifi*WifiDevice*mac_address*)
        *(.text.*esp_wifi*wifi*embassy*impl*embassy_net_driver*)

        *(.text.*drop_in_place*esp_wifi*compat*timer_compat*Timer*)

        /* esp-wifi-sys */
        *(.text.net80211_printf)    /* phy_printf, pp_printf */


    } > ROTEXT