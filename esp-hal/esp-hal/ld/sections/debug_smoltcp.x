.smoltcp : ALIGN(4)
{
    *(.text.*smoltcp*iface*interface*InterfaceInner*check_ip_addrs*)
    *(.text.*smoltcp*iface*neighbor*Cache*new*)
    *(.text.*smoltcp*iface*route*Routes*new*)
    *(.text.*smoltcp*iface*route*Routes*add_default_ipv4_route*)
    *(.text.*smoltcp*iface*route*Routes*remove_default_ipv4_route*)
    *(.text.*smoltcp*iface*socket_set*SocketSet*add*put*)
    *(.text.*smoltcp*iface*socket_set*SocketSet*remove*)
    *(.text.*smoltcp*socket*dhcpv*Socket*reset*)
    *(.text.*smoltcp*wire*ipv4*Cidr*new*)

} > ROTEXT
