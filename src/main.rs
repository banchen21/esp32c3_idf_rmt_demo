use std::{thread, time::Duration};

use anyhow::Result;
use esp_idf_hal::{
    prelude::Peripherals,
    rmt::{
        FixedLengthSignal, PinState, Pulse, PulseTicks, RxRmtConfig, RxRmtDriver, TxRmtConfig,
        TxRmtDriver,
    },
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use log::{debug, info};
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let rx_config = RxRmtConfig::new().clock_divider(1);
    let channel = peripherals.rmt.channel1;
    let pin = peripherals.pins.gpio4;
    let mut rx = RxRmtDriver::new(channel, pin, &rx_config, 128)?;
    rx.start()?;

    // Prepare the config. 中文：准备配置
    let config = TxRmtConfig::new().clock_divider(1);
    // Retrieve the output pin and channel from peripherals. 中文：获取输出引脚和通道
    let channel = peripherals.rmt.channel0;
    let pin = peripherals.pins.gpio18;
    // Create an RMT transmitter. 中文：创建一个 RMT 传输器
    let mut tx = TxRmtDriver::new(channel, pin, &config)?;

    thread::spawn(move || {
        loop {
            // Receive the signal. 中文：接收信号
            let buf = &mut [(Pulse::zero(), Pulse::zero())];
            while let Ok(signal) = rx.receive(buf, 10) {
                info!("Received signal: {:?}", signal);
            }
        }
    });

    thread::spawn(move || {
        loop {
            // Prepare signal pulse signal to be sent. 中文：准备要发送的信号脉冲
            let low = Pulse::new(PinState::Low, PulseTicks::new(10).unwrap());
            let high = Pulse::new(PinState::High, PulseTicks::new(10).unwrap());
            let mut signal = FixedLengthSignal::<2>::new();
            signal.set(0, &(low, high)).unwrap();
            signal.set(1, &(high, low)).unwrap();
            // Transmit the signal. 中文：发送信号
            tx.start(signal).unwrap();
        }
    });

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
