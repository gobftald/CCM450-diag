.app : ALIGN(4)
{
    *(.text.main)

    *(.text.*embassy_executor*raw*TaskStorage*poll*)

    *(.text.*ecu_via_elm327*ecu*__server_task*__server_task_inner_function*closure*ecu_reply*)

    *(.text.*ecu_via_elm327*adapter*adapter_implementation*Adapter*wait_AT_prompt*closure*)

    *(.text.*ecu_via_elm327*adapter*adapter_implementation*decode_err_status*)

    *(.text.*ecu_via_elm327*adapter*utils*from_ascii_bytes_to_u8*)
    *(.text.*ecu_via_elm327*adapter*utils*from_u8_to_ascii_bytes*)
    *(.text.*ecu_via_elm327*adapter*utils*from_ascii_bytes_to_u16*)
    *(.text.*ecu_via_elm327*adapter*utils*from_u16_to_ascii_bytes*)

    *(.text.*ecu_via_elm327*udp*send_to*closure*)

} > ROTEXT
