// 1
use core::task::Context;

// 3
use embassy_net_driver::{Capabilities, Checksum, Driver};
use smoltcp::phy::{self, Medium};

// 7
pub(crate) struct DriverAdapter<'d, 'c, T>
where
    T: Driver,
{
    // must be Some when actually using this to rx/tx
    pub cx: Option<&'d mut Context<'c>>,
    pub inner: &'d mut T,
    pub medium: Medium,
}

// 17
impl<'d, 'c, T> phy::Device for DriverAdapter<'d, 'c, T>
where
    T: Driver,
{
    /// Get a description of device capabilities.
    fn capabilities(&self) -> phy::DeviceCapabilities {
        fn convert(c: Checksum) -> phy::Checksum {
            match c {
                Checksum::Both => phy::Checksum::Both,
                Checksum::Tx => phy::Checksum::Tx,
                Checksum::Rx => phy::Checksum::Rx,
                Checksum::None => phy::Checksum::None,
            }
        }

        let caps: Capabilities = self.inner.capabilities();
        let mut smolcaps = phy::DeviceCapabilities::default();

        smolcaps.max_transmission_unit = caps.max_transmission_unit;
        smolcaps.max_burst_size = caps.max_burst_size;
        smolcaps.medium = self.medium;
        smolcaps.checksum.ipv4 = convert(caps.checksum.ipv4);
        smolcaps.checksum.tcp = convert(caps.checksum.tcp);
        smolcaps.checksum.udp = convert(caps.checksum.udp);
        #[cfg(feature = "proto-ipv4")]
        {
            smolcaps.checksum.icmpv4 = convert(caps.checksum.icmpv4);
        }
        #[cfg(feature = "proto-ipv6")]
        {
            smolcaps.checksum.icmpv6 = convert(caps.checksum.icmpv6);
        }

        smolcaps
    }
}
