use super::*;
use crate::adapter::{Adapter, Adapters};

const SECRET_KEY: u32 = 0x1EC3;

enum State {
    Disconnected,
    Connected,
}

/// A Keihin KMSK16 ECU
pub struct ECU<'a> {
    adapter: Adapter<'a>,
    state: State,
}

impl<'a> ECU<'a> {
    pub fn new(adapter: Adapter<'a>) -> Self {
        Self {
            adapter,
            state: State::Disconnected,
        }
    }

    async fn write(&mut self, request: &[u8]) -> Result<usize, EcuError> {
        self.adapter
            .write(request)
            .await
            .map_err(EcuError::AdapterError)
    }
}

impl<'a> Ecu for ECU<'a> {
    // we cannot shortcut async-await with future since we convert errors
    async fn connect(&mut self) -> Result<(), EcuError> {
        match self.state {
            State::Disconnected => {
                trace!("#### adapter.connect()");
                self.adapter
                    .connect(SECRET_KEY)
                    .await
                    .map_err(EcuError::AdapterError)?;
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
            .map_err(EcuError::AdapterError)
    }
}
