pub struct Protocol {
    format: u8,
    target: u8,
    source: u8,
}

#[derive(Clone, Copy)] 
pub enum ServiceId {
    ClearDiagnosticInformation = 0x14,
    ReadDiagnosticTroubleCodesByStatus = 0x18,
    ReadDataByCommonId = 0x22,
    TesterPresent = 0x3e,
    StartCommunication = 0x81,
}

use super::{Protocols, EcuError};

impl Protocol {
    pub fn new (format: u8, target: u8, source: u8) -> Self {
        Self { format, target, source }
    }
}

impl Protocols for Protocol {
    fn format_request(&self, service_id: u8, param: &[u8], buf: &mut [u8]) -> usize {
        let plen = param.len();
        buf[0] = self.format + plen as u8 + 1;
        buf[1] = self.target;
        buf[2] = self.source;
        buf[3] = service_id;
        
        if plen > 0 {
            buf[4..].copy_from_slice(param);
        }

        let mut checksum: u8 = 0;
        for byte in buf[..plen + 4].iter_mut() {
            checksum += *byte;
        }
        buf[plen + 4] = checksum;

        plen + 5
    }

    fn parse_response<'a>(&self, service_id: u8, response: &[u8]) -> Result<usize, EcuError> {
        Ok(0)
    }
}
