// use const_array_concat::ConcatableArray;
use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use log::{error, info, warn};
use trouble_host::prelude::*;
use usbd_hid::descriptor::{KeyboardUsage, MediaKey};

use crate::hid_report::{
    HID_DESCRIPTOR, KEYBOARD_REPORT_SIZE, KeyboardOutputReport, KeyboardReport,
    MEDIA_KEYBOARD_REPORT_SIZE, MOUSE_REPORT_SIZE, MediaKeyboardReport, MouseReport,
};

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 4; // Signal + att

// GATT Server definition
#[gatt_server]
struct Server {
    battery_service: BatteryService,
    hid_service: HidService,
}

#[gatt_service(uuid = service::HUMAN_INTERFACE_DEVICE)]
pub(crate) struct HidService {
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03], permissions(encrypted))]
    pub(crate) hid_info: [u8; 4],

    #[characteristic(uuid = "2a4b", read, value = HID_DESCRIPTOR, permissions(encrypted))]
    pub(crate) report_map: [u8; HID_DESCRIPTOR.len()],

    #[characteristic(uuid = "2a4c", write_without_response, permissions(encrypted))]
    pub(crate) hid_control_point: u8,

    #[characteristic(
        uuid = "2a4e",
        read,
        write_without_response,
        value = 1,
        permissions(encrypted)
    )]
    pub(crate) protocol_mode: u8,

    #[descriptor(uuid = "2908", read = encrypted, value = [1u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify, permissions(encrypted))]
    pub(crate) input_keyboard: [u8; KEYBOARD_REPORT_SIZE],

    #[descriptor(uuid = "2908", read = encrypted, value = [2u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify, permissions(encrypted))]
    pub(crate) input_mouse: [u8; MOUSE_REPORT_SIZE],

    #[descriptor(uuid = "2908", read = encrypted, value = [3u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify, permissions(encrypted))]
    pub(crate) input_media_keyboard: [u8; MEDIA_KEYBOARD_REPORT_SIZE],

    #[descriptor(uuid = "2908", read = encrypted, value = [1u8, 2u8])]
    #[characteristic(
        uuid = "2a4d",
        read,
        write,
        write_without_response,
        permissions(encrypted)
    )]
    pub(crate) output_keyboard: [u8; 1],
}

/// Battery service
#[gatt_service(uuid = service::BATTERY)]
struct BatteryService {
    /// Battery Level
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, name = "hello", read, value = "Battery Level", type = &'static str)]
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify, value = 10, permissions(authenticated))]
    level: u8,
    #[characteristic(uuid = "408813df-5dd4-1f87-ec11-cdb001100000", write, read, notify)]
    status: bool,
}

/// Run the BLE stack.
pub async fn run<C>(controller: C)
where
    C: Controller,
{
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x08, 0x05, 0xe4, 0xff]);
    info!("Our address = {}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(address)
        .set_io_capabilities(IoCapabilities::DisplayOnly)
        .build();

    let runner = stack.runner();
    let mut peripheral = stack.peripheral();

    info!("Starting advertising and GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "TrouBLE",
        appearance: &appearance::human_interface_device::GENERIC_HUMAN_INTERFACE_DEVICE,
    }))
    .unwrap();

    let _ = join(ble_task(runner), async {
        loop {
            match advertise("Trouble Example", &mut peripheral, &server).await {
                Ok(conn) => {
                    conn.raw().set_bondable(true).unwrap();
                    // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                    let a = gatt_events_task(&server, &conn);
                    let b = custom_task(&server, &conn, &stack);
                    let c = hid_report(&server, &conn);
                    // run until any task ends (usually because the connection has been closed),
                    // then return to advertising state.
                    select(a, select(b, c)).await;
                    info!("Connection dropped");
                }
                Err(e) => {
                    panic!("[adv] error: {:?}", e);
                }
            }
        }
    })
    .await;
}

/// This is a background task that is required to run forever alongside any other BLE tasks.
///
/// ## Alternative
///
/// If you didn't require this to be generic for your application, you could statically spawn this with i.e.
///
/// ```rust,ignore
///
/// #[embassy_executor::task]
/// async fn ble_task(mut runner: Runner<'static, SoftdeviceController<'static>>) {
///     runner.run().await;
/// }
///
/// spawner.must_spawn(ble_task(runner));
/// ```
async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            panic!("[ble_task] error: {:?}", e);
        }
    }
}

/// Stream Events until the connection closes.
///
/// This function will handle the GATT events and process them.
/// This is how we interact with read and write requests.
async fn gatt_events_task(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, DefaultPacketPool>,
) -> Result<(), Error> {
    let output_keyboard = server.hid_service.output_keyboard;
    let reason =
        loop {
            match conn.next().await {
                GattConnectionEvent::Disconnected { reason } => break reason,
                GattConnectionEvent::PairingComplete {
                    security_level,
                    bond,
                } => {
                    info!("[gatt] pairing complete: {:?}", security_level);
                    if let Some(_bond) = bond {
                        info!("Bond information stored");
                    }
                }
                GattConnectionEvent::PairingFailed(err) => {
                    error!("[gatt] pairing error: {:?}", err);
                }
                GattConnectionEvent::PassKeyDisplay(key) => {
                    info!("PassKeyDisplay {:?}", key);
                }
                GattConnectionEvent::PassKeyConfirm(key) => {
                    info!("PassKeyConfirm {:?}", key);
                    conn.pass_key_confirm().unwrap();
                }
                GattConnectionEvent::PassKeyInput => {
                    info!("PassKeyInput");
                }
                GattConnectionEvent::Gatt { event } => {
                    match &event {
                        GattEvent::Read(event) => {
                            if event.handle() == output_keyboard.handle {
                                let value = conn.get(&output_keyboard);
                                info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                            }
                        }
                        GattEvent::Write(event) => {
                            if event.handle() == output_keyboard.handle {
                                event.with_data(|offset, data| {
                                info!(
                                    "[gatt] Write Event to Level Characteristic at {}: {:?} {:?}",
                                    offset, data, KeyboardOutputReport::new(data[0]).leds()
                                )
                            });
                            }
                        }
                        GattEvent::NotAllowed(event) => {
                            info!(
                                "[gatt] Disallowed GATT request to handle: {:?}",
                                event.handle()
                            );
                        }
                        _ => (),
                    }

                    match event.accept() {
                        Ok(reply) => reply.send().await,
                        Err(e) => warn!("[gatt] error sending response: {:?}", e),
                    }
                }
                _ => {} // ignore other Gatt Connection Events
            }
        };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}

/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::IncompleteServiceUuids16(&[
                service::BATTERY.to_le_bytes(),
                service::HUMAN_INTERFACE_DEVICE.to_le_bytes(),
            ]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

/// Example task to use the BLE notifier interface.
/// This task will notify the connected central of a counter value every 2 seconds.
/// It will also read the RSSI value every 2 seconds.
/// and will stop when the connection is closed by the central or an error occurs.
async fn custom_task<C: Controller, P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    stack: &Stack<'_, C, P>,
) {
    let mut tick: u8 = 0;
    let level = server.battery_service.level;
    loop {
        tick = tick.wrapping_add(1);
        // info!("[custom_task] notifying connection of tick {}", tick);
        if level.notify(conn, &tick, true).await.is_err() {
            info!("[custom_task] error notifying connection");
            break;
        };
        // read RSSI (Received Signal Strength Indicator) of the connection.
        if let Ok(rssi) = conn.raw().rssi(stack).await {
            // info!("[custom_task] RSSI: {:?}", rssi);
        } else {
            info!("[custom_task] error getting RSSI");
            break;
        };
        Timer::after_secs(2).await;
    }
}

pub static KEY_EVENTS: Channel<CriticalSectionRawMutex, bool, 32> = Channel::new();

async fn hid_report<P: PacketPool>(server: &Server<'_>, conn: &GattConnection<'_, '_, P>) {
    loop {
        let pressed = KEY_EVENTS.receive().await;
        info!(
            "[hid] reading {}",
            if pressed { "pressed" } else { "released" }
        );

        let path = 2;
        let res = match path {
            1 => report_kb(server, conn, pressed).await,
            2 => report_mouse(server, conn, pressed).await,
            3 => report_media_kb(server, conn, pressed).await,
            _ => Ok(()),
        };
        if res.is_err() {
            info!("[hid] error notifying connection");
            break;
        }
    }
}

async fn report_kb<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    pressed: bool,
) -> Result<(), ()> {
    let input = &server.hid_service.input_keyboard;
    let mut report = KeyboardReport::default();
    if pressed {
        report.set(KeyboardUsage::KeyboardBb, true);
        report.set(KeyboardUsage::KeyboardLeftShift, true);
    }

    if input.notify(conn, &report.serialize(), true).await.is_err() {
        info!("[hid] error notifying connection");
        return Err(());
    }

    Ok(())
}

async fn report_mouse<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    pressed: bool,
) -> Result<(), ()> {
    if !pressed {
        return Ok(());
    }

    let input = &server.hid_service.input_mouse;
    let report = MouseReport::new(10, 0);

    if input.notify(conn, &report.serialize(), true).await.is_err() {
        info!("[hid] error notifying connection");
        return Err(());
    }

    let report = MouseReport::default();

    if input.notify(conn, &report.serialize(), true).await.is_err() {
        return Err(());
    }

    Ok(())
}

async fn report_media_kb<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    pressed: bool,
) -> Result<(), ()> {
    let input = &server.hid_service.input_media_keyboard;
    let mut report = MediaKeyboardReport::default();
    if pressed {
        report.set(MediaKey::VolumeDecrement);
    }

    if input.notify(conn, &report.serialize(), true).await.is_err() {
        return Err(());
    }

    Ok(())
}
