use super::*;
use crate::ecu_protocol::Protocol;

enum State {
    Disconnected,
}

/// A Keihin KMSK16 ECU
pub struct ECU {
    protocol: Protocol,
    state: State,
}

impl Ecu for ECU {
    fn new() -> Self {
        Self {
            protocol: Protocol,
            state: State::Disconnected,
        }
    }

    fn connect(&self) {}

    fn process_request(&self, req: Request) -> Result<Response, Error> {
        Ok(Response::Ok)
    }

    fn process_response(&self) {}
}
