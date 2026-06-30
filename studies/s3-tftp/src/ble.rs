use trouble_host::prelude::{
    HostResources, DefaultPacketPool, ExternalController, Host,
    gatt_server, gatt_service, FromGatt, Uuid, GapConfig, PeripheralConfig,
    appearance, AdStructure, Advertisement, AdvertisementParameters,
    IoCapabilities, GattConnectionEvent, GattEvent,
};

//use trouble_host::prelude::*;

use crate::gps::{GPS_DATA, GPS_UPDATED};

// =====================================================================
// NORDIC UART SERVICE (NUS) GATT SERVER GENERATION
// =====================================================================
pub const NUS_SERVICE_UUID: Uuid = Uuid::new_long([
    0x6e, 0x40, 0x00, 0x01, 0xb5, 0xa3, 0xf3, 0x93, 
    0xe0, 0xa9, 0xe5, 0x0e, 0x24, 0xdc, 0xca, 0x9e
]);
pub const NUS_TX_CHAR_UUID: Uuid = Uuid::new_long([
    0x6e, 0x40, 0x00, 0x03, 0xb5, 0xa3, 0xf3, 0x93, 
    0xe0, 0xa9, 0xe5, 0x0e, 0x24, 0xdc, 0xca, 0x9e
]);

#[gatt_server]
pub struct NusGattServer {
    pub uart: NusService,
}

#[gatt_service(uuid = NUS_SERVICE_UUID)]
pub struct NusService {
    #[characteristic(uuid = NUS_TX_CHAR_UUID, notify)]
     pub tx: u8,
}

#[embassy_executor::task]
pub async fn ble_ssp_task(
    radio_controller: &'static esp_radio::Controller<'static>,
    bt_periheral: esp_hal::peripherals::BT<'static>
) {
    // get GPS_UPDATES watch receiver
    let mut gps_updated = unwrap!(GPS_UPDATED.receiver());

    trace!("*** BLE: Initializing serial profile emulation...");

    let transport  = unwrap!(
        esp_radio::ble::controller::BleConnector::new(
            radio_controller,
            bt_periheral,
            Default::default(),
        )   
    );

    // we are sending only with 1Hz, so 2 slots enough
    let controller =
        ExternalController::<_, 2>::new(transport);

    // Allocate static resources required to hold connection tracking
    let resources = crate::mk_static!(
        HostResources<DefaultPacketPool, 1, 1, 1>, 
        HostResources::new()
    );

    let stack = trouble_host::new(controller, resources)
        .set_io_capabilities(IoCapabilities::NoInputNoOutput);

    let Host {
        mut peripheral,
        central,
        mut runner,
        ..
    } = stack.build();

    let server =
        unwrap!(
            NusGattServer::new_with_config(
                GapConfig::Peripheral(
                    PeripheralConfig {
                        name: "CCM-GP450",
                        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
                    }
                )
            )
        );

    let mut adv_data = [0u8; 31];
    let adv_len = match AdStructure::encode_slice(
        &[
            // GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED
            AdStructure::Flags(0x06),
            AdStructure::CompleteLocalName(b"CCM-GP450"),
        ],
        &mut adv_data,
    ) {
        Ok(len) => len,
        Err(err) => {
            error!("*** BLE AdStructure Encoding Fail: {:?}", err);
            panic!("Failed to encode BLE advertising slice");
        }
    };

    // Cooperative background driver loop executing lower link layers concurrently
    let ble_radio_runner = async { 
        if let Err(err) = runner.run().await {
            error!("*** BLE Link Layer failure: {:?}", err);
        }
    };

    // Dynamic virtual serial transmission pipeline
    let gps_spp_loop = async {
        loop {
            trace!("*** BLE SPP: Advertising serial coordinates link over the air...");

            let adv_params = AdvertisementParameters::default();
            let payload = Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..adv_len],
                scan_data: &[],
            };

            let advertising_result =
                peripheral.advertise(&adv_params, payload).await;

            let advertiser =
                match advertising_result {
                    Ok(adv) => adv,
                    Err(err) => {
                        error!("*** BLE Advertisement Fail: {:?}", err);
                        embassy_time::Timer::after_secs(1).await;
                        continue;
                    }
                };
            
            let raw_connection_result = advertiser.accept().await;

            let raw_connection
                = match  raw_connection_result {
                Ok(conn) => conn,
                Err(err) => {
                    error!("*** BLE Accept Fail: {:?}", err);
                    embassy_time::Timer::after_secs(1).await;
                    continue;
                }
            };

            // Upgrade the raw link-layer connection into a GATT-aware one bound to our server.
            let connection = match raw_connection.with_attribute_server(&server) {
                Ok(conn) => conn,
                Err(err) => {
                    error!("*** BLE Attribute Server Bind Fail: {:?}", err);
                    embassy_time::Timer::after_secs(1).await;
                    continue;
                }
            };

            trace!("*** BLE SPP: Phone connected successfully via GATT!");

            let gatt_events_task = async {
                loop {
                    debug!("*** BLE SPP: gatt_events_task loop");

                    match connection.next().await {
                        GattConnectionEvent::Disconnected { reason } => {
                            trace!("*** BLE SPP: Disconnected: {:?}", defmt::Debug2Format(&reason));
                            break;
                        }
                        GattConnectionEvent::Gatt { event } => {
                            match &event {
                                GattEvent::Read(_) | GattEvent::Write(_) => {
                                    // your existing read/write handling, if any, goes here
                                }
                                _ => {}
                            }

                            // Explicitly send the reply now rather than relying on drop.
                            if let Err(err) = event.accept() {
                                warn!("*** BLE: Failed to accept GATT event: {:?}", err);
                            }
                        }
                        _ => {}
                    }
                }
            };

            let send_loop = async {
                loop {
                    debug!("*** BLE SPP: send_loop loop");
                    gps_updated.changed().await;
                    trace!("*** BLE SPP: Sending 1Hz coordinate packet line...");
                    
                    if let Err(err) = server.uart.tx.notify(&connection, &b"haho"[0]).await {
                        warn!("*** BLE SPP: Notify failed (client probably unsubscribed/dropped): {:?}", err);
                        return; // bail out of send_loop, gatt_events_task will catch the disconnect
                    }
                }
            };

            // Run protocol handling and sending side-by-side; whichever finishes first
            // (disconnect detected, or a notify failure) ends this connection's lifetime.
            embassy_futures::select::select(gatt_events_task, send_loop).await;

            trace!("*** BLE SPP: Link dropped cleanly. Resetting state tracking...");
        }
    };

    embassy_futures::select::select(ble_radio_runner, gps_spp_loop).await;
}
