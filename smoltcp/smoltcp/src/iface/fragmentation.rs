// 255
pub(crate) struct FragmentsBuffer {
    //#[cfg(feature = "proto-sixlowpan")]
    //pub decompress_buf: [u8; MAX_DECOMPRESSED_LEN],
    #[cfg(feature = "_proto-fragmentation")]
    pub assembler: PacketAssemblerSet<FragKey>,

    #[cfg(feature = "_proto-fragmentation")]
    pub reassembly_timeout: Duration,
}

#[cfg(not(feature = "_proto-fragmentation"))]
// 267
pub(crate) struct Fragmenter {}

#[cfg(not(feature = "_proto-fragmentation"))]
// 270
impl Fragmenter {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
