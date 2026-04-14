use super::{EcuApi, Error, EcuApiError};
use crate::adapter::{ Adapters, Adapter};
use crate::protocol::{Protocols, Protocol, ServiceId};

const FORMAT_BYTE: u8 = 0x80;               // physical addressing
const TARGET_ADDR: u8 = 0x12;               // ECU address
const SOURCE_ADDR: u8 = 0xF1;               // Tester address
const FAST_INIT_HALF_PERIOD: u32 = 25_000;  // 25ms in usec
const SECRET_KEY: u16 = 0x1EC3;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EcuError(pub u8);

#[derive(Clone, Copy, PartialEq)]
enum State {
    Disconnected,
    Connected,
    ReadIds(usize, usize),
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
            let size = self.poll(ServiceId::StartCommunication as u8, &[], &mut buf)
                .await
                // ignore Keyword Bytes
                .map(|mut size | { size -= 2; size } )?;
    
            self.state = State::Connected;
            Ok(size)
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

    async fn clear_dtc(&mut self) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            let mut buf = [0u8; 8];
            let size = self.poll(ServiceId::ClearDiagnosticInformation as u8, &[0xff, 0xff], &mut buf)
                .await
                .map(|mut size | { size -= 2; size } )?;
            Ok(size)
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn security_access(&mut self,) -> Result<usize,Error> {
        if self.state != State::Disconnected {
            let mut buf = [0u8; 8];
            // request seed
            loop {
                self.poll(ServiceId::SecurityAccess as u8, &[0x03], &mut buf).await?;
                // I found/know secret key only for even seeds
                if ((buf[5] as u16 * 256) + (buf[6] as u16)) % 2 == 0 {
                    break
                }
            }

            // calculate response
            let key = ((buf[5] as u16 * 256) + (buf[6] as u16)) * SECRET_KEY;

            // send key
            let size = self.poll(
                ServiceId::SecurityAccess as u8,
                &[0x04, (key / 256) as u8, (key % 256) as u8],
                &mut buf)
                .await
                // ignore positive response in response
                .map(|mut size | { size -= 1; size } )?;
            Ok(size)
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    fn read_ids(&mut self,response: &mut [u8]) -> usize {
        static mut SAVE_STATE: (State, bool) = (State::Disconnected, false);
        // Safety: we use this 'static mut' only in this function
        unsafe {
            // save State only the first call
            if !SAVE_STATE.1 {
                SAVE_STATE.0 = self.state;
                SAVE_STATE.1 = true;
            }
        }

        let buf_len = (response.len() - 4) / 2;

        let (mut counter, index) =  if let State::ReadIds(counter, index) = self.state {
            (counter, index)
        } else {
            (IDS_ARRAY.len(), 0)
        };

        let len = if buf_len < counter{
            buf_len
        } else {
            counter
        };

        counter -= len;
        if counter > 0 {
            self.state = State::ReadIds(counter, index + len);
        } else {
            unsafe {
                self.state = SAVE_STATE.0;
                // reset State saving mechanism after the last call
                SAVE_STATE.1 = false;
            }
        }

        response[0] = (len / 256) as u8;
        response[1] = (len % 256) as u8;
        response[2] = (counter / 256) as u8;
        response[3] = (counter % 256) as u8;

        let mut j = 0usize;
        for i in index..index + len {
            response[4 + (j * 2)] = (IDS_ARRAY[i].0 / 256) as u8; 
            response[4 + (j * 2) + 1] = (IDS_ARRAY[i].0 % 256) as u8;
            j += 1;
        }

        len * 2 + 4
    }

    fn get_ids_description(&mut self, id: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        response[0] = id[0];
        response[1] = id[1];

        let idx = (id[0] as u16) * 256 + (id[1] as u16);

        if let Some(&(_, descr)) = IDS_ARRAY.into_iter().find(|(i, _)| *i == idx) {
            unsafe {
                core::ptr::copy_nonoverlapping(descr.as_ptr(), response[2..].as_mut_ptr(), descr.len());
                Ok(descr.len() + 2)
            }
        } else {
            Err(EcuApiError::InvalidId.into())
        }
    }

    async fn read_data(&mut self, ids: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            self.poll(ServiceId::ReadDataByCommonId as u8, &ids[..2], response).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

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

const IDS_ARRAY: &[(u16, &[u8])] = &[
    (0x0000u16, b"Absolute throttle position sensor voltage - THAD"),

    (0x0001u16, b"Absolute throttle position sensor - THM/80 deg *100 %"),

    (0x0002u16, b"Intake manifold abs pressure voltage (bank1) - voltage PM1"),

    (0x0003u16, b"Intake manifold absolute pressure (bank1) - PM1"),

    (0x0006u16, b"Battery voltage // directly at the AD converter"),

    (0x0007u16, b"Battery voltage (scaling)"),

    (0x0008u16, b"Engine coolant temperature voltage"),

    (0x0009u16, b"Engine coolant temperature - TW"),

    (0x0010u16, b"Intake air temperature voltage"),

    (0x0011u16, b"Intake air temperature - TA"),

    (0x0012u16, b"HEGO sensor1 voltage (bank1) - VHG1)"),

    (0x0018u16, b"2nd throttle actuator on voltage"),

    (0x0040u16, b"Neutral switch - NEUTRAL"),

    (0x0053u16, b"MAP SW // current state of the sports switch input - MAPCHG"),

    (0x0060u16, b"Fuel pump relay - FLPR"),

    (0x0064u16, b"Heated exhaust gas oxygen sensor Heater1 - HG1HT)"),

    (0x0100u16, b"Engine rpm (2byte value - high byte/low byte) - NE)"),

    (0x0102u16, b"Short term fuel trim - bank1 (2byte - high/low byte) - MHG1"),

    (0x0107u16, b"Calculated load Value - CLV=(PM1M/10103 mba)r * 100%"),

    (0x0110u16, b"Injector1 ON Time (bank1) - TIOUT1)"),

    (0x0120u16, b"Ignition timing1 (bank1) - IGAB1"),

    (0x0130u16, b"Ignition coil dwell time1 (bank1) - IGDWELL"),

    (0x0140u16, b"THREF"),

    (0x0142u16, b"MREFHG1"),

    (0x0143u16, b"MREFHG1ID"),

    (0x0149u16, b"MRFH1DEC"),

    (0x0170u16, b"2nd throttle curr angl - (STHAD-STHADL)/(STHADH/STHADL)*100%"),

    (0x0171u16, b"2nd throttle target angle - STHTRG"),

    (0x0172u16, b"2nd throttle ADLL - STHADLL"),

    (0x0173u16, b"2nd throttle ADHH - STDHADHH"),

    (0x0174u16, b"2nd throttle steph - STHSTEPH"),

    (0x0185u16, b"Purge valve duty cycle"),

    (0x0500u16, b"RaceMAP SW // Sport switch has been activated"),

    (0x0501u16, b"High REV counter"),

    (0x0502u16, b"High REV counter area 1"),

    (0x0503u16, b"High REV counter area 2"),

    (0x0504u16, b"High REV counter area 3"),

    (0x0505u16, b"High REV counter area 4"),

    (0x0506u16, b"High REV counter area 5"),

    (0x0507u16, b"High REV counter area 6"),

    (0x0508u16, b"High REV counter area 7"),

    (0x0509u16, b"High REV counter area 8"),

    (0x0510u16, b"High REV counter area 9"),
];
