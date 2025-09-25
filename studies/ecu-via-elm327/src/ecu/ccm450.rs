use super::*;
use crate::{
    ecu_adapter::{Adapter, Adapters},
    ecu_protocol::Protocol,
};

enum State {
    Disconnected,
    Connected,
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

    async fn write(&mut self, request: &[u8]) -> Result<usize, EcuError> {
        self.adapter
            .write(request)
            .await
            .map_err(EcuError::CommunicationError)
    }
}

impl<'a> Ecu for ECU<'a> {
    // we cannot shortcut async-await with future since we convert errors
    async fn connect(&mut self) -> Result<(), EcuError> {
        match self.state {
            State::Disconnected => {
                trace!("#### adapter.connect()");
                self.adapter
                    .connect()
                    .await
                    .map_err(EcuError::CommunicationError)?;
                //self.state = State::Connected;
                Ok(())
            }
            _ => Err(EcuError::InconsystentRequest),
        }
    }

    // we cannot shortcut async-await with future since here we convert errors
    async fn reply(&mut self, reply: &mut [u8]) -> Result<usize, EcuError> {
        self.adapter
            .read(reply)
            .await
            .map_err(EcuError::CommunicationError)
    }
}
