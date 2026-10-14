#![no_std]
#![no_main]

use bt_hci::controller::ExternalController;
use defmt::{info};
use esp_radio::{ble::controller::BleConnector};
use trouble_host::prelude::*;
use embassy_futures::join::join;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

#[gatt_server]
struct Server {
}

#[embassy_executor::task]
pub async fn run_bluetooth(transport: BleConnector<'static>) {
    info!("running bluetooth");
    let ble_controller = ExternalController::<_, 1>::new(transport);
    let mut resources: HostResources<_, DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let stack = trouble_host::new(ble_controller, &mut resources).build();
    let runner = stack.runner();
    let mut peripheral = stack.peripheral();

    let mut server = match Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Weather-Station",
        appearance: &appearance::sensor::MULTI_SENSOR,
    })) {
        Ok(server) => server,
        Err(e) => {
            panic!("{:?}", e);
        },
    };
    
    let _ = join(ble_task(runner), handle_bluetooth(&mut peripheral, &mut server)).await;
}

async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            let e = defmt::Debug2Format(&e);
            panic!("{:?}", e);
        }
    }
}

async fn handle_bluetooth<'values, 'server, C>(peripheral: &mut Peripheral<'values, C, DefaultPacketPool>, server: &'server Server<'values>)
where C: Controller,
{
    loop {
        match advertise("Trouble Example", peripheral, &server).await {
            Ok(conn) => {
                // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                loop {
                    let event = conn.next().await;
                    info!("{:?}", event);
                }
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}

async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0_u8; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::IncompleteServiceUuids16(&[[0x0f, 0x18]]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    info!("awaiting advertiser");
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("advertising bluetooth");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("bluetooth connection established");
    Ok(conn)
}