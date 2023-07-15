use bincode::{config, decode_from_slice};
use blucher_data::commands::Command;

use blucher_data::wifi::{TCP_ADDRESS, TCP_BUFFER_SIZE, TCP_PORT};
use cyw43_pio::PioSpi;
use defmt::*;

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Stack, StackResources};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0};
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::Duration;
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
pub async fn wifi_system(
    spawner: Spawner,
    ssid: &'static str,
    pass: &'static str,
    pwr_pin: PIN_23,
    cs_pin: PIN_25,
    pio_pin: PIO0,
    dio_pin: PIN_24,
    clk_pin: PIN_29,
    dma_pin: DMA_CH0,
    command_channel: Sender<'static, ThreadModeRawMutex, Command, 8>,
) {
    info!("Initializing wifi system");

    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs-cli download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs-cli download 43439A0_clm.bin --format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(pwr_pin, Level::Low);
    let cs = Output::new(cs_pin, Level::High);
    let mut pio = Pio::new(pio_pin);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        dio_pin,
        clk_pin,
        dma_pin,
    );

    let state = make_static!(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
        .await;

    // let config = Config::dhcpv4(Default::default());
    // Use a link-local address for communication without DHCP server
    let config = Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(
            embassy_net::Ipv4Address::new(
                TCP_ADDRESS[0],
                TCP_ADDRESS[1],
                TCP_ADDRESS[2],
                TCP_ADDRESS[3],
            ),
            16,
        ),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let stack = &*make_static!(Stack::new(
        net_device,
        config,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    control.start_ap_wpa2(ssid, pass, 5).await;

    let mut rx_buffer = [0; TCP_BUFFER_SIZE];
    let mut tx_buffer = [0; TCP_BUFFER_SIZE];
    let mut buf = [0; TCP_BUFFER_SIZE];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        control.gpio_set(0, false).await;
        info!("Listening on TCP:{}", TCP_PORT);
        if let Err(e) = socket.accept(TCP_PORT).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        let (mut reader, mut writer) = socket.split();

        // let incoming_task = handle_incoming_messages(reader);
        let incoming_task = async {
            loop {
                let lenght = match reader.read(&mut buf).await {
                    Ok(0) => {
                        warn!("EOF reached over TCP, disconnecting");
                        break;
                    }
                    Ok(lenght) => lenght,
                    Err(err) => {
                        error!("Error reading data over TCP: {:?}, disconnecting", err);
                        break;
                    }
                };

                debug!("Received {} bytes over TCP", lenght);

                let Ok( (decoded, _) ) = decode_from_slice::<Command, _>(&buf[..lenght], config::standard()) else {
                    error!("Error decoding message received over TCP.");
                    continue;
                };

                debug!("Received message over TCP: {}", decoded);

                match command_channel.try_send(decoded) {
                    Ok(()) => {
                        debug!("Command is being processed");
                        break;
                    },
                    Err(_) => {
                        error!("Error passing command to command queue, command lost");
                    },
                };
            }
        };

        let outgoing_task = async {
            let buf = [0; TCP_BUFFER_SIZE];
            loop {
                let length = match writer.write(&buf).await {
                    Ok(length) => length,
                    Err(err) => {
                        error!("Error writing data over TCP: {:?}, disconnecting", err);
                        break;
                    }
                };

                debug!("Sent {} bytes over TCP", length);
            }
        };

        select(incoming_task, outgoing_task).await;

        info!("TCP Connection closed");

        // loop {
        //     let n = match socket.read(&mut buf).await {
        //         Ok(0) => {
        //             warn!("EOF reached over TCP");
        //             break;
        //         }
        //         Ok(n) => n,
        //         Err(e) => {
        //             error!("Error reading data over TCP: {:?}", e);
        //             break;
        //         }
        //     };
        //
        //     let Ok( (decoded, length) ) = decode_from_slice::<Command, _>(&buf[..n], config::standard()) else {
        //         error!("Error decoding message received over TCP.");
        //         break;
        //     };
        //
        //     debug!("Received {} bytes over TCP", length);
        //
        //     match socket.write_all(&buf[..n]).await {
        //         Ok(()) => {}
        //         Err(e) => {
        //             warn!("write error: {:?}", e);
        //             break;
        //         }
        //     };
        // }
    }
}
