.smoltcp : ALIGN(4)
{
    *(.text.*smoltcp*socket*Socket*poll_at*)

    *(.text.*smoltcp*socket*dhcpv4*Socket*parse_ack*)
    *(.text.*smoltcp*socket*dhcpv4*Socket*process*)
    *(.text.*smoltcp*socket*dhcpv4*Socket*reset*)
    
    *(.text.*smoltcp*socket*udp*Socket*process*)

    *(.text.*smoltcp*socket*waker*WakerRegistration*register*)


    *(.text.*smoltcp*storage*packet_buffer*PacketBuffer*enqueue*)

    *(.text.*smoltcp*storage*ring_buffer*RingBuffer*dequeue_one_with*)
    *(.text.*smoltcp*storage*ring_buffer*RingBuffer*dequeue_many_with*)
    *(.text.*smoltcp*storage*ring_buffer*RingBuffer*enqueue_many*)


    *(.text.*smoltcp*iface*socket_set*SocketSet*add*put*)

    *(.text.*smoltcp*iface*route*Routes*remove_default_ipv4_route*)
    *(.text.*smoltcp*iface*route*Routes*lookup*)

    *(.text.*smoltcp*iface*neighbor*Cache*lookup*)

    *(.text.*smoltcp*iface*interface*ipv4*impl*smoltcp*iface*interface*InterfaceInner*is_unicast_v4*)
    *(.text.*smoltcp*iface*interface*ipv4*impl*smoltcp*iface*interface*InterfaceInner*icmpv4_reply*)
            
    *(.text.*smoltcp*iface*interface*InterfaceInner*dispatch_ip*)
    *(.text.*smoltcp*iface*interface*InterfaceInner*has_neighbor*)
    *(.text.*smoltcp*iface*interface*InterfaceInner*is_broadcast*)
    *(.text.*smoltcp*iface*interface*InterfaceInner*in_same_network*)
    *(.text.*smoltcp*iface*interface*InterfaceInner*route*)

    *(.text.*smoltcp*iface*interface*Interface*update_ip_addrs*)


    *(.text.*smoltcp*wire*dhcpv4*Repr*buffer_len*)

    *(.text.*smoltcp*wire*udp*Packet*fill_checksum*)
    *(.text.*smoltcp*wire*udp*Packet*payload*)
    *(.text.*smoltcp*wire*udp*Repr*parse*)
    
    *(.text.*smoltcp*wire*ipv4*Repr*emit*)
    *(.text.*smoltcp*wire*ipv4*Packet*new_checked*)
    
    *(.text.*smoltcp*wire*ip*checksum*data*)
    *(.text.*smoltcp*wire*ip*checksum*pseudo_header*)

} > ROTEXT
