#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
use atat_examples::common;

use std::process::exit;

use atat::{asynch::AtatClient, AtatIngress, Buffers, Config, DefaultDigester, Ingress};
use embedded_io::adapters::FromTokio;
use tokio_serial::SerialStream;

const INGRESS_BUF_SIZE: usize = 1024;
const URC_CAPACITY: usize = 128;
const URC_SUBSCRIBERS: usize = 3;

#[tokio::main]
async fn main() -> ! {
    env_logger::init();

    static BUFFERS: Buffers<common::Urc, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS> =
        Buffers::<common::Urc, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>::new();

    let (reader, writer) = SerialStream::pair().expect("Failed to create serial pair");

    let (ingress, mut client) = BUFFERS.split(
        FromTokio::new(writer),
        DefaultDigester::<common::Urc>::default(),
        Config::default(),
    );

    tokio::spawn(ingress_task(ingress, FromTokio::new(reader)));

    let mut state: u8 = 0;
    loop {
        // These will all timeout after 1 sec, as there is no response
        match state {
            0 => {
                client.send(&common::general::GetManufacturerId).await.ok();
            }
            1 => {
                client.send(&common::general::GetModelId).await.ok();
            }
            2 => {
                client.send(&common::general::GetSoftwareVersion).await.ok();
            }
            3 => {
                client.send(&common::general::GetWifiMac).await.ok();
            }
            _ => exit(0),
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        state += 1;
    }
}

async fn ingress_task<'a>(
    mut ingress: Ingress<
        'a,
        DefaultDigester<common::Urc>,
        common::Urc,
        INGRESS_BUF_SIZE,
        URC_CAPACITY,
        URC_SUBSCRIBERS,
    >,
    mut reader: FromTokio<SerialStream>,
) -> ! {
    ingress.read_from(&mut reader).await
}
