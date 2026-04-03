use super::{EcuApi, Error, EcuApiError};
use crate::adapter::{ Adapters, Adapter};
use crate::protocol::{Protocols, Protocol, ServiceId};

const FORMAT_BYTE: u8 = 0x80;       // physical addressing
const TARGET_ADDR: u8 = 0x12;       // ECU address
const SOURCE_ADDR: u8 = 0xF1;       // Tester address

const SECRET_KEY: u32 = 0x1EC3;

#[derive(PartialEq)]
enum State {
    Disconnected,
    Connected,
}

/// Keihin KMSK16 ECU
#[allow(clippy::upper_case_acronyms)]
pub struct ECU<'a> {
    adapter: Adapter<'a>,
    protocol: Protocol,
    state: State,
}

impl<'a> ECU<'a> {
    
    pub fn new(adapter: Adapter<'a>) -> Self {
        Self {
            adapter,
            // Protocol specification handled by feature gated module
            protocol: Protocol::new(FORMAT_BYTE, TARGET_ADDR, SOURCE_ADDR),
            state: State::Disconnected,
        }
    }

    async fn poll(&mut self, sid: ServiceId, param: &[u8], reply: &mut [u8]) -> Result<usize, Error> {
        let mut buf = [0u8; 8];
        let len = self.protocol.format_request(sid as u8, param, &mut buf);

        trace!("#### poll transmit: {:x}", &buf[..len]);
        self.adapter.transmit(&buf[..len]).await.map_err(Error::AdapterError)?;
        let len = self.adapter.receive(reply).await.map_err(Error::AdapterError)?;
        trace!("#### poll receive: {:x}", reply);

        self.protocol.parse_response(sid as u8, &mut reply[..len]).map_err(Error::EcuError)
    }
}

#[cfg(feature = "l9637")]
impl<'a> EcuApi for ECU<'a> {
    async fn connect(&mut self) -> Result<(), Error> {
        if self.state == State::Disconnected {
            self.poll(ServiceId::StartCommunication, &[], &mut []).await?;
            self.state = State::Connected;
            Ok(())
        } else {
            Err(EcuApiError::AlreadyConnected.into())
        }
    }

    async fn read_data(&mut self, ids: &[u8], reply: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            self.poll(ServiceId::ReadDataByCommonId, &ids[..1], reply).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn read_dtc(&mut self, reply: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            // 0x02 - Request 2 byte hex DTC, 
            self.poll(ServiceId::ReadDiagnosticTroubleCodesByStatus, &[0x02, 0xFF], reply).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn clear_dtc(&mut self) -> Result<(), Error> {
        if self.state != State::Disconnected { 
            self.poll(ServiceId::ClearDiagnosticInformation, &[], &mut []).await.map(|_| ())
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn raw_request(&mut self, request: &[u8], reply: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected { 
            Ok(0)
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    // we cannot shortcut async-await with future since here we convert errors
    async fn reply(&mut self, reply: &mut [u8]) -> Result<usize, Error> {
        self.adapter.receive(reply).await.map_err(Error::AdapterError)
    }
}

// For ELM327 adapter we cannot use kwp2000 protocol, since programming of this
// adapter needs sending proprietary byte sequences to its UART interface.
// Using function names similar to kwp2000 ServiceId we only group these API
// programming sequences into blocks sending the corresponding kwp2000 messages.
#[cfg(feature = "elm327")]
impl<'a> EcuApi for ECU<'a> {
    async fn connect(&mut self) -> Result<(), EcuError> {
        self.adapter
            .connect(SECRET_KEY)
            .await
            .map_err(EcuError::AdapterError)?;
        self.state = State::Connected;
        Ok(())
    }

    async fn read_data(&mut self, ids: &[u8], reply: &mut [u8]) -> Result<usize, EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .read_data_by_common_id(ids, reply)
                .await
                .map_err(EcuError::AdapterError),
            _ => Err(EcuError::NotConnected),
        }
    }

    async fn read_dtc(&mut self, reply: &mut [u8]) -> Result<usize, EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .read_diagnostic_trouble_codes_by_status(reply)
                .await
                .map_err(EcuError::AdapterError),
            _ => Err(EcuError::NotConnected),
        }
    }

    async fn clear_dtc(&mut self) -> Result<(), EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .clear_diagnostic_information()
                .await
                .map_err(EcuError::AdapterError),
            _ => Err(EcuError::NotConnected),
        }
    }

    async fn raw_request(&mut self, request: &[u8], reply: &mut [u8]) -> Result<usize, EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .raw_request(request, reply)
                .await
                .map_err(EcuError::AdapterError),
            _ => Err(EcuError::NotConnected),
        }
    }

    // we cannot shortcut async-await with future since here we convert errors
    async fn reply(&mut self, reply: &mut [u8]) -> Result<usize, EcuError> {
        self.adapter
            .read(reply)
            .await
            .map_err(EcuError::AdapterError)
    }
}
