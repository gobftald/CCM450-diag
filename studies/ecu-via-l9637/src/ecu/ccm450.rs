use super::{EcuApi, Error, EcuApiError};
use crate::adapter::{ Adapters, Adapter};
use crate::protocol::{Protocols, Protocol, ServiceId};

const FORMAT_BYTE: u8 = 0x80;               // physical addressing
const TARGET_ADDR: u8 = 0x12;               // ECU address
const SOURCE_ADDR: u8 = 0xF1;               // Tester address
const FAST_INIT_HALF_PERIOD: u32 = 25_000;  // 25ms in usec
const SECRET_KEY: u32 = 0x1EC3;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EcuError(pub u8);

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

    async fn poll(&mut self, sid: u8, param: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        let len = self.protocol.format_request(sid, param, response);
        self.adapter.transmit(&mut response[..len]).await.map_err(Error::AdapterError)?;

        let len = self.adapter.receive(response).await.map_err(Error::AdapterError)?;
        self.protocol.parse_response(sid, &mut response[..len]).map_err(Error::ProtocolError)
    }
}

#[cfg(feature = "l9637")]
impl<'a> EcuApi for ECU<'a> {
    async fn connect(&mut self) -> Result<usize, Error> {
        if self.state == State::Disconnected {
            // Fast Init
            self.adapter.tx.send_break(FAST_INIT_HALF_PERIOD);
            esp_hal::rom::ets_delay_us(FAST_INIT_HALF_PERIOD);

            let mut buf = [0u8; 8];
            let ret = self.poll(ServiceId::StartCommunication as u8, &[], &mut buf).await;
    
            self.state = State::Connected;
            ret
        } else {
            Err(EcuApiError::AlreadyConnected.into())
        }
    }

    async fn tester_present(&mut self) {
        let mut buf = [0u8; 8];
        // 0x01 is the only arg which was accepted, but cannot cancel echo/response
        self.poll(ServiceId::TesterPresent as u8, &[0x01], &mut buf).await.ok();
    }

    async fn raw_request(&mut self, request: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            self.poll(request[0], &request[1..], response).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn read_dtc(&mut self, response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            // 0x02 - Request all 2 byte hex DTCs, 
            self.poll(ServiceId::ReadDiagnosticTroubleCodesByStatus as u8, &[0x02, 0xFF, 0xFF], response).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn clear_dtc(&mut self) -> Result<(), Error> {
        if self.state != State::Disconnected { 
            self.poll(ServiceId::ClearDiagnosticInformation as u8, &[], &mut []).await.map(|_| ())
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn read_data(&mut self, ids: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            self.poll(ServiceId::ReadDataByCommonId as u8, &ids[..1], response).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    // we cannot shortcut async-await with future since here we convert errors
    async fn response(&mut self, response: &mut [u8]) -> Result<usize, Error> {
        self.adapter.receive(response).await.map_err(Error::AdapterError)
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

    async fn read_data(&mut self, ids: &[u8], response: &mut [u8]) -> Result<usize, EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .read_data_by_common_id(ids, response)
                .await
                .map_err(EcuError::AdapterError),
            _ => Err(EcuError::NotConnected),
        }
    }

    async fn read_dtc(&mut self, response: &mut [u8]) -> Result<usize, EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .read_diagnostic_trouble_codes_by_status(response)
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

    async fn raw_request(&mut self, request: &[u8], response: &mut [u8]) -> Result<usize, EcuError> {
        match self.state {
            State::Connected => self
                .adapter
                .raw_request(request, response)
                .await
                .map_err(EcuError::AdapterError),
            _ => Err(EcuError::NotConnected),
        }
    }

    // we cannot shortcut async-await with future since here we convert errors
    async fn response(&mut self, response: &mut [u8]) -> Result<usize, EcuError> {
        self.adapter
            .read(response)
            .await
            .map_err(EcuError::AdapterError)
    }
}
