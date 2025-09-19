use super::*;
use crate::{
    ecu_adapter::{Adapter, Adapters},
    ecu_protocol::Protocol,
};

enum State {
    Disconnected,
}

/// A Keihin KMSK16 ECU
pub struct ECU<'a> {
    adapter: Adapter<'a>,
    protocol: Protocol,
    state: State,
}

impl<'a> ECU<'a> {
    pub fn new(adapter: Adapter<'a>) -> Self {
        Self {
            adapter,
            protocol: Protocol,
            state: State::Disconnected,
        }
    }

    async fn write(&mut self, request: &[u8]) -> Result<(), EcuError> {
        if let Ok(sent) = self.adapter.write_async(request).await {
            if sent != request.len() {
                Err(EcuError::CommunicationFailed)
            } else {
                Ok(())
            }
        } else {
            Err(EcuError::CommunicationFailed)
        }
    }
}

impl<'a> Ecus for ECU<'a> {
    async fn request(&mut self, request: Request) -> Result<(), EcuError> {
        match request {
            Request::Connect => self.write(b"ATZ\r").await,
            _ => Err(EcuError::InvalidRequest),
        }
    }

    async fn response(&mut self, response: &mut [u8]) -> Result<usize, EcuError> {
        trace!("ecu/ccm450: read_async(response).await");
        let res = self.adapter
            .read_async(response)
            .await
            .map_err(|_| EcuError::CommunicationFailed);
        trace!("ecu/ccm450: read_async(response).await awaken");
        res
    }
}
