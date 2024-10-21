use anyhow::Result;
use esp_idf_hal::{
    delay::FreeRtos,
    prelude::Peripherals,
    rmt::{
        config::{CarrierConfig, DutyPercent},
        FixedLengthSignal, PinState, Pulse, PulseTicks, Receive, RmtReceiveConfig,
        RmtTransmitConfig, RxRmtDriver, TxRmtDriver,
    },
};
use log::{debug, info};
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take()?;

    let carrier_config = CarrierConfig::new().duty_percent(DutyPercent::new(50)?);
    let rx_config = RmtReceiveConfig::new()
        .carrier(Some(carrier_config))
        .idle_threshold(65535);
    let mut rx = RxRmtDriver::new(
        peripherals.rmt.channel2,
        peripherals.pins.gpio2,
        &rx_config,
        1000,
    )?;
    rx.start()?;
    let mut tx = TxRmtDriver::new(
        peripherals.rmt.channel0,
        peripherals.pins.gpio4,
        &RmtTransmitConfig::new().carrier(Some(carrier_config)),
    )?;

    let _ = std::thread::Builder::new()
        .stack_size(10000)
        .spawn(move || loop {
            let mut pulses: [(Pulse, Pulse); 1000] = [(Pulse::zero(), Pulse::zero()); 1000];
            let receive = rx.receive(&mut pulses, 0).unwrap();
            if let Receive::Read(length) = receive {
                let pulses: &[(Pulse, Pulse)] = &pulses[..length];
                let mut byte_data: Vec<u8> = Vec::new();
                for (_lenght, (p1, p2)) in pulses.iter().enumerate() {
                    // println!(
                    //     "{:?},{:?},{:?},{:?}",
                    //     p1.ticks, p1.pin_state, p2.ticks, p2.pin_state
                    // );
                    if p1.pin_state == PinState::Low
                        && p2.pin_state == PinState::High
                        && in_range(p2.ticks.ticks(), 350, 700)
                    {
                        byte_data.push(0x0);
                    } else if p1.pin_state == PinState::Low
                        && p2.pin_state == PinState::High
                        && in_range(p2.ticks.ticks(), 350, 1850)
                    {
                        byte_data.push(0x1);
                    }
                }
                let byte_data: Vec<u8> = bits_to_bytes(&byte_data);
                if byte_data.len() == 0 {
                    break;
                }
                for bytedat in &byte_data {
                    print!("{:02X} ", bytedat);
                }
                println!();
                R05dDecode::decode(byte_data);
                println!("length: {}", length);
            }
            FreeRtos::delay_ms(500);
        });

    let _ = std::thread::spawn(move || loop {
        FreeRtos::delay_ms(3000);
        // 关机
        send_wave_code(&mut tx, 0xB2, 0xBF, 0x00).unwrap();
    });
    loop {
        FreeRtos::delay_ms(3000);
    }
}

fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    let mut byte_data: Vec<u8> = Vec::new();

    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            byte |= bit << (7 - i); // 将比特左移到正确的位置
        }
        byte_data.push(byte);
    }

    byte_data
}

// 判断某个数字是否在范围内
fn in_range(num: u16, min: u16, max: u16) -> bool {
    if num >= min && num <= max {
        return true;
    }
    false
}
// 引导码
fn send_header_code(tx: &mut TxRmtDriver) -> Result<()> {
    let p1 = Pulse::new(PinState::High, PulseTicks::new(4400).unwrap());
    let p2 = Pulse::new(PinState::Low, PulseTicks::new(4400).unwrap());
    let mut s = FixedLengthSignal::<1>::new();
    s.set(0, &(p1, p2)).unwrap();
    tx.start(s).unwrap();
    Ok(())
}

// 数据码0
fn send_0_code(tx: &mut TxRmtDriver) -> Result<()> {
    let p1 = Pulse::new(PinState::High, PulseTicks::new(520).unwrap());
    let p2 = Pulse::new(PinState::Low, PulseTicks::new(540).unwrap());
    let mut s = FixedLengthSignal::<1>::new();
    s.set(0, &(p1, p2)).unwrap();
    tx.start(s).unwrap();
    Ok(())
}

// 数据码1
fn send_1_code(tx: &mut TxRmtDriver) -> Result<()> {
    let p1 = Pulse::new(PinState::High, PulseTicks::new(520).unwrap());
    let p2 = Pulse::new(PinState::Low, PulseTicks::new(1600).unwrap());
    let mut s = FixedLengthSignal::<1>::new();
    s.set(0, &(p1, p2)).unwrap();
    tx.start(s).unwrap();
    Ok(())
}

// 分隔码
fn send_stop_code(tx: &mut TxRmtDriver) -> Result<()> {
    let p1 = Pulse::new(PinState::High, PulseTicks::new(520).unwrap());
    let p2 = Pulse::new(PinState::Low, PulseTicks::new(5220).unwrap());
    let mut s = FixedLengthSignal::<1>::new();
    s.set(0, &(p1, p2)).unwrap();
    tx.start(s).unwrap();
    Ok(())
}

// 发送字节码
fn send_byte_code(tx: &mut TxRmtDriver, byte: u8) -> Result<()> {
    // 遍历字节的每一位，从最高位到最低位
    for i in (0..8).rev() {
        let bit = (byte >> i) & 0x01; // 提取当前位
        if bit == 0 {
            send_0_code(tx)?; // 发送数据码0
        } else {
            send_1_code(tx)?; // 发送数据码1
        }
    }
    Ok(())
}

// 完整波形
fn send_wave_code(tx: &mut TxRmtDriver, a: u8, b: u8, c: u8) -> Result<()> {
    send_header_code(tx)?;
    send_byte_code(tx, a)?;
    send_byte_code(tx, !a)?;
    send_byte_code(tx, b)?;
    send_byte_code(tx, !b)?;
    send_byte_code(tx, c)?;
    send_byte_code(tx, !c)?;
    send_stop_code(tx)?;

    send_header_code(tx)?;
    send_byte_code(tx, a)?;
    send_byte_code(tx, !a)?;
    send_byte_code(tx, b)?;
    send_byte_code(tx, !b)?;
    send_byte_code(tx, c)?;
    send_byte_code(tx, !c)?;
    send_stop_code(tx)?;

    //  FF D5 66 00 10 00 4B
    // 11111111110101010110011000000000000100000000000001001000
    send_header_code(tx)?;
    send_byte_code(tx, 0xD5)?;
    send_byte_code(tx, 0x66)?;
    send_byte_code(tx, 0x00)?;
    send_byte_code(tx, 0x10)?;
    send_byte_code(tx, 0x00)?;
    send_byte_code(tx, 0x4B)?;
    send_stop_code(tx)?;

    Ok(())
}

// 风速
enum Speed {
    // 自动
    Auto = 5,
    // 低风
    Low = 4,
    // 中风
    Middle = 2,
    // 高风
    High = 1,
}

impl Speed {
    fn as_u8(data: u8) -> Speed {
        println!("风速: {}", data);
        match data {
            5 => Speed::Auto,
            4 => Speed::Low,
            2 => Speed::Middle,
            1 => Speed::High,
            _ => Speed::Auto,
        }
    }
}

// 模式
enum Mode {
    // 自动
    Auto = 2,
    // 制冷
    Cool = 0,
    // 抽湿，送风
    Humidify = 1,
    // 制热
    Heat = 3,
}

impl Mode {
    fn as_u8(data: u8) -> Mode {
        println!("模式: {}", data);
        match data {
            2 => Mode::Auto,
            0 => Mode::Cool,
            1 => Mode::Humidify,
            3 => Mode::Heat,
            _ => Mode::Auto,
        }
    }
}

// 温度
enum R05dTemp {
    T17 = 0,
    T18 = 1,
    T19 = 3,
    T20 = 2,
    T21 = 6,
    T22 = 7,
    T23 = 5,
    T24 = 4,
    T25 = 12,
    T26 = 13,
    T27 = 9,
    T28 = 8,
    T29 = 10,
    T30 = 11,
}

impl R05dTemp {
    fn as_u8(data: u8) -> R05dTemp {
        println!("温度: {}", data);
        match data {
            0 => R05dTemp::T17,
            1 => R05dTemp::T18,
            3 => R05dTemp::T19,
            2 => R05dTemp::T20,
            6 => R05dTemp::T21,
            7 => R05dTemp::T22,
            5 => R05dTemp::T23,
            4 => R05dTemp::T24,
            12 => R05dTemp::T25,
            13 => R05dTemp::T26,
            9 => R05dTemp::T27,
            8 => R05dTemp::T28,
            10 => R05dTemp::T29,
            11 => R05dTemp::T30,
            _ => R05dTemp::T30,
        }
    }
}

// 解码
struct R05dDecode;
impl R05dDecode {
    fn decode(byte_data: Vec<u8>) {
        // 编码格式为 L A A' B B' C C' S L A A' B B' C C'
        println!(
            "A: {:02X}, A`: {:02X}, B: {:02X} B`: {:02X}, C: {:02X}, C`: {:02X}",
            byte_data[0], byte_data[1], byte_data[2], byte_data[3], byte_data[4], byte_data[5],
        );
        if byte_data[2] == 0x7B && byte_data[4] == 0xE0 {
            println!("关机")
        } else if byte_data[2] == 0xF5 && byte_data[4] == 0x04 {
            println!("自动扫风");
        } else if byte_data[2] == 0xF5 && byte_data[4] == 0x05 {
            println!("手动扫风");
        } else {
            //风速
            let wind = (byte_data[2] >> 5) & 0x7;
            //模式
            let mode = (byte_data[4] >> 2) & 0x3;
            // 温度
            let temp = (byte_data[4] >> 4) & 0xf;
            // 风速
            match Speed::as_u8(wind) {
                Speed::Auto => {
                    println!("自动,");
                }
                Speed::Low => {
                    println!("低风,");
                }
                Speed::Middle => {
                    println!("中风,");
                }
                Speed::High => {
                    println!("高风,");
                }
            }

            match Mode::as_u8(mode) {
                Mode::Cool => {
                    println!("制冷,");
                }
                Mode::Humidify => {
                    println!("抽湿,");
                }
                Mode::Heat => {
                    println!("制热,");
                }
                Mode::Auto => {
                    println!("自动,");
                }
            }
            match R05dTemp::as_u8(temp) {
                R05dTemp::T17 => {
                    println!("温度：17°C");
                }
                R05dTemp::T18 => {
                    println!("温度：18°C");
                }
                R05dTemp::T19 => {
                    println!("温度：19°C");
                }
                R05dTemp::T20 => {
                    println!("温度：20°C");
                }
                R05dTemp::T21 => {
                    println!("温度：21°C");
                }
                R05dTemp::T22 => {
                    println!("温度：22°C");
                }
                R05dTemp::T23 => {
                    println!("温度：23°C");
                }
                R05dTemp::T24 => {
                    println!("温度：24°C");
                }
                R05dTemp::T25 => {
                    println!("温度：25°C");
                }
                R05dTemp::T26 => {
                    println!("温度：26°C");
                }
                R05dTemp::T27 => {
                    println!("温度：27°C");
                }
                R05dTemp::T28 => {
                    println!("温度：28°C");
                }
                R05dTemp::T29 => {
                    println!("温度：29°C");
                }
                R05dTemp::T30 => {
                    println!("温度：30°C");
                }
            }
        }
    }
}
