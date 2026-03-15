.embassy : ALIGN(4)
{
    *(.text.*embassy_executor*raw*poll_exited*)

    /* it implements application code/logic
    *(.text.*embassy_executor*raw*TaskStorage*poll*)
    */
    
    *(.text.*embassy_executor*raw*util*UninitCell*write_in_place*)
    *(.text.*embassy_executor*raw*SyncExecutor*spawn*)
    
    *(.text.*embassy_executor*raw*waker*drop*)
    *(.text.*embassy_executor*raw*waker*wake*)
    *(.text.*embassy_executor*raw*waker*clone*)

    *(.text.*embassy_executor*raw*Executor*spawn*)

    *(.text.*drop_in_place*Result*embassy_sync*zerocopy_channel*Channel*embassy_sync*blocking_mutex*raw*NoopRawMutex*ecu_via_elm327*ChannelItem*)
    
    *(.text.*embassy_sync*waitqueue*atomic_waker*GenericAtomicWaker*register*)

    *(.text.*embassy_sync*zerocopy_channel*Sender*send_done*)
    *(.text.*embassy_sync*zerocopy_channel*Receiver*receive_done*)

    *(.text.*embassy_net*driver_util*DriverAdapter*as*smoltcp*phy*Device*transmit*)

    *(.text.*embassy_net*Inner*apply_static_config*)

    *(.text.*embassy_time*timer*Timer*as*core*future*future*Future*poll*)

    *(.text._embassy_time_schedule_wake)
    *(.text.__embassy_time_queue_item_from_waker)

    *(.text.*embassy_executor*raw*state*State*update*)
    *(.text.*embassy_sync*blocking_mutex*Mutex*lock*)

} > ROTEXT