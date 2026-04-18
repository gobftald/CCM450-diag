use embassy_futures::select::{Either,select};

use super::{EcuApi, Error, EcuApiError, IdType};
use crate::adapter::{ Adapters, Adapter};
use crate::protocol::{Protocol, ProtocolError, Protocols, ServiceId};

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

    async fn poll(&mut self, sid: u8, param: &[u8], response: &mut [u8], timeout_ms: u32) -> Result<usize, Error> {
        let len = self.protocol.format_request(sid, param, response);
        self.adapter.transmit(&mut response[..len]).await.map_err(Error::AdapterError)?;

        if timeout_ms > 0 {
            trace!("timeout");
            match select(
                self.adapter.receive(response),
                embassy_time::Timer::after(embassy_time::Duration::from_millis(timeout_ms as u64))
            ).await {
                Either::First(ret) => {
                    let len = ret.map_err(Error::AdapterError)?;
                    self.protocol.parse_response(sid, &mut response[..len]).map_err(Error::ProtocolError)
                },
                Either::Second(_) => {
                    Err(ProtocolError::Timeout.into())
                }
            }
        } else {
            let len = self.adapter.receive(response).await.map_err(Error::AdapterError)?;
            self.protocol.parse_response(sid, &mut response[..len]).map_err(Error::ProtocolError)
        }
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
            let size = self.poll(
                ServiceId::StartCommunication as u8,
                &[],
                &mut buf,
                0
            )
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
        self.poll(ServiceId::TesterPresent as u8, &[0x01], &mut buf, 0).await.ok();
    }

    async fn raw_request(&mut self, request: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            self.poll(request[0], &request[1..], response, 200).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn read_dtc(&mut self, response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            // 0x02 - Request all 2 byte hex DTCs, 
            self.poll(
                ServiceId::ReadDiagnosticTroubleCodesByStatus as u8,
                &[0x02, 0xFF, 0xFF],
                response,
                0
            ).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn clear_dtc(&mut self) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            let mut buf = [0u8; 8];
            let size = self.poll(
                ServiceId::ClearDiagnosticInformation as u8,
                &[0xff, 0xff],
                &mut buf,
                0
            )
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
                self.poll(ServiceId::SecurityAccess as u8, &[0x03], &mut buf, 0).await?;
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
                &mut buf,
                0
            )
                .await
                // ignore positive response in response
                .map(|mut size | { size -= 1; size } )?;
            Ok(size)
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    fn get_ids(&mut self, id_type: IdType, response: &mut [u8]) -> usize {        
        let id_array = match id_type {
            IdType::ReadData => DATA_IDS,
            IdType::StartRoutine => START_ROUTINE_IDS,
            IdType::StopRoutine => STOP_ROUTINE_IDS,
        };

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
            (id_array.len(), 0)
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
            response[4 + (j * 2)] = (id_array[i].0 / 256) as u8; 
            response[4 + (j * 2) + 1] = (id_array[i].0 % 256) as u8;
            j += 1;
        }

        len * 2 + 4
    }

    fn get_id_description(
        &mut self,
        id_type: IdType,
        id: &[u8],
        response: &mut [u8]
    ) -> Result<usize, Error> {
        let descr_array = match id_type {
            IdType::ReadData => DATA_IDS,
            IdType::StartRoutine => START_ROUTINE_IDS,
            IdType::StopRoutine => STOP_ROUTINE_IDS,
        };

        response[0] = id[0];
        response[1] = id[1];

        let idx= (id[0] as u16) * 256 + (id[1] as u16);

        if let Some(&(_, descr)) = descr_array.into_iter().find(|(i, _)| *i == idx) {
            unsafe {
                core::ptr::copy_nonoverlapping(descr.as_ptr(), response[2..].as_mut_ptr(), descr.len());
                Ok(descr.len() + 2)
            }
        } else {
            Err(EcuApiError::InvalidId.into())
        }
    }

    async fn read_data(&mut self, id: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        if self.state != State::Disconnected {
            self.poll(ServiceId::ReadDataByCommonId as u8, id, response, 0).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn start_routine(&mut self, arg: &[u8],response: &mut [u8]) -> Result<usize,Error> {
        if self.state != State::Disconnected {
            self.poll(ServiceId::StartRoutineByLocalIdentifier as u8, arg, response, 200).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    async fn stop_routine(&mut self, arg: &[u8],response: &mut [u8]) -> Result<usize,Error> {
        if self.state != State::Disconnected {
            self.poll(ServiceId::StopRoutineByLocalIdentifier as u8, arg, response, 0).await
        } else {
            Err(EcuApiError::NotConnected.into())
        }
    }

    // not used yet
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

const DATA_IDS: &[(u16, &[u8])] = &[
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

    (0x2500u16, b"Read backup of the supplier information field"),

    (0x2501u16, b"Read Flash Times"),

    (0x2502u16, b"Read HW Reference"),

    (0x2503u16, b"Read Program Reference"),

    (0x2504u16, b"Read Data Reference"),

    (0x2506u16, b"Read Flash Block Length"),
    ];

const START_ROUTINE_IDS: &[(u16, &[u8])] = &[
    (0x0102, b"CheckCodingChecksum - Program (Applization Code)"),
    (0x0104, b"CheckCodingChecksum - Data (Calibration Code)"),
    
    // 0x0a (but we need to define u16)
    (0x000a, b"CheckProgrammingStatus - ignore 0x00, one byte only: 0x0a"),

    // Modern ECUs are "learning" machines. They constantly adjust parameters to compensate
    // for wear and tear. This routine wipes that memory.
    //
    // What it resets: Fuel trim values (long-term/short-term), throttle body alignment positions,
    // and sensor offset calibrations.
    //
    // When to use it: After replacing a major engine component (like an O2 sensor, fuel injector,
    //
    // or throttle body). It forces the ECU to start learning from a "clean slate" rather than trying
    // to apply old, incorrect compensation values to new hardware.
    (0x91e2, b"study ctrl data, breakdown info, all reference reset operation"),

    // Study Control Data: Resets the "statistical" data the car tracks about the driver
    //
    // Breakdown Information: Clears historical environmental data related to faults. In some
    // systems, this resets "first-occurrence" timers or permanent internal counters
    // that standard DTC (Diagnostic Trouble Code) clearing might not touch.
    //
    // Reference Reset: It essentially tells the ECU: "Forget everything that has happened
    // since you left the factory assembly line."
    //
    // When to use it: Typically used at the very end of the production line or when a "remanufactured"
    // ECU is being installed to ensure it doesn't carry over data from the previous vehicle.
    (0xa000, b"All reference reset operation - (Delete Adaption)"),
    // Warning: Using these two above via StartRoutineByLocalIdentifier without following up with the correct 
    // re-learning procedure (like a specific idling sequence or driving cycle) can sometimes cause the vehicle 
    // to run poorly or throw "Configuration Not Performed" faults.
    
    (0xa401, b"Ignition test 'bank1'"),
    (0xa501, b"Injector test 'bank1'"),
    (0xa600, b"Purge control valve ON/OFF test"),
    (0xa700, b"Fuel pump relay ON/OFF test"),
    (0xa701, b"Fuel pump relay operation ON"),
    (0xa900, b"2nd throttle valve control stepper motor test"),
    (0xaf01, b"HEGO sensor operation (heating) ON 'bank1'"),
];

const STOP_ROUTINE_IDS: &[(u16, &[u8])] = &[
    (0xa701, b"Fuel pump relay operation OFF"),
    (0xaf01, b"HEGO sensor operation (heating) OFF 'bank1'"),
];