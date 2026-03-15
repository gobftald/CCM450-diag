.esp-rtos : ALIGN(4)
{
    *(.text.*esp_rtos*with_deadline*)
    *(.text.*esp_rtos*now*)

    *(.text.*esp_rtos*esp_radio*queue*QueueInner*try_dequeue*)
    *(.text.*esp_rtos*esp_radio*queue*QueueInner*try_enqueue*)
    *(.text.*esp_rtos*esp_radio*queue*QueueInner*get_mut*)    

    *(.text.*esp_rtos*run_queue*Priority*new*)
    *(.text.*esp_rtos*run_queue*RunQueue*remove*)

    *(.text.*esp_rtos*semaphore*SemaphoreInner*try_take*)

    *(.text.*esp_rtos*scheduler*SchedulerState*sleep_task_until*)
    *(.text.*esp_rtos*scheduler*SchedulerState*delete_marked_tasks*)

    *(.text.*esp_rtos*embassy*Executor*run_inner*)
    *(.text.*esp_rtos*embassy*Executor*run*)
    *(.text.*esp_rtos*embassy*ThreadFlag*new*)

    *(.text.*esp_rtos*timer*TimeDriver*arm_next_wakeup*)

    *(.text.*esp_rtos*esp_radio*timer_queue*Timer*as*esp_radio_rtos_driver*timer*TimerImplementation*create*closure*)
    *(.text.*esp_rtos*esp_radio*timer_queue*timer_task*)
    *(.text.*esp_rtos*esp_radio*timer_queue*TimerQueueInner*enqueue*)

    *(.text.*esp_rtos*task*arch_specific*idle_hook*)
    *(.text.*esp_rtos*task*Task*as*core*ops*drop*Drop*drop*)
    *(.text.*esp_rtos*task*task_wrapper*)
    *(.text.*esp_rtos*task*schedule_task_deletion*)

    *(.text.esp_rtos_queue_create)
    *(.text.esp_rtos_queue_receive)
    *(.text.esp_rtos_queue_send_to_back)
    *(.text.esp_rtos_queue_send_to_front)
    *(.text.esp_rtos_semaphore_give)
    *(.text.esp_rtos_semaphore_take)
    *(.text.esp_rtos_timer_delete)


    *(.text.*esp_rtos*wait_queue*WaitQueue*remove*)
    *(.text.*esp_rtos*timer*TimerQueue*remove*)
    *(.text.*esp_rtos*wait_queue*WaitQueue*notify*)

    
} > ROTEXT
