.libcore : ALIGN(4)
{
    *(.text.misc_nvs_load)
    *(.text.misc_nvs_deinit)
    *(.text.misc_nvs_init)

    /* libmesh.a(mesh_parent.o) */
    *(.text.mesh_sta_auth_expire_time)

} > ROTEXT