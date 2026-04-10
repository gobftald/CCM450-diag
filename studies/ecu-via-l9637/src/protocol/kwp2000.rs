use crate::ecu::EcuError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ProtocolError {
    InvalidChecksum,
    EcuError(EcuError)
}

impl From<ProtocolError> for u8 {
    fn from(value: ProtocolError) -> Self {
        match value {
            ProtocolError::InvalidChecksum => 0,
            ProtocolError::EcuError(_) => 1,
        }
    }
}

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
    SecurityAccess = 0x27,
    TesterPresent = 0x3e,
    StartCommunication = 0x81,
}

use super::Protocols;

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
            buf[4..plen + 4].copy_from_slice(param);
        }

        let mut checksum: u8 = 0;
        for byte in buf[..plen + 4].iter() {
            checksum += *byte;
        }
        buf[plen + 4] = checksum;

        plen + 5
    }

    fn parse_response(&self, service_id: u8, response: &mut [u8]) -> Result<usize, ProtocolError> {
        let len = response.len();
        // check checksum
        let mut checksum: u8 = 0;
        for byte in response[..len - 1].iter() {
            checksum += *byte;
        }
        if checksum != response[response.len() - 1] {
            return Err(ProtocolError::InvalidChecksum)
        }

        // check for negative response and send error coming from ecu
        if service_id + 0x40 != response[3] {
            return Err(ProtocolError::EcuError(EcuError(response[5])))
        }

        // response[4..len-1]
        response.copy_within(4..len -1, 0);
        Ok(len - 5)
    }
}
