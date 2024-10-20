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
                for (lenght, (p1, p2)) in pulses.iter().enumerate() {
                    // println!("lenght: {lenght}| p1: {p1:?}| p2: {p2:?}");
                    if p1.pin_state == PinState::Low
                        && p2.pin_state == PinState::High
                        && in_range(p1.ticks.ticks(), 4200, 4600)
                        && in_range(p2.ticks.ticks(), 4200, 4600)
                    {
                        // println!("L");
                    } else if p1.pin_state == PinState::Low
                        && p2.pin_state == PinState::High
                        && in_range(p2.ticks.ticks(), 350, 700)
                    {
                        byte_data.push(0x0);
                    } else if p1.pin_state == PinState::Low
                        && p2.pin_state == PinState::High
                        && in_range(p2.ticks.ticks(), 350, 1850)
                    {
                        byte_data.push(0x1);
                    } else if p1.pin_state == PinState::Low
                        && p2.pin_state == PinState::High
                        && in_range(p2.ticks.ticks(), 420, 5400)
                    {
                        // println!("S");
                    }
                }
                let byte_data = bits_to_bytes(&byte_data);
                // 打印转换后的字节数组，使用16进制格式
                println!("Received: {byte_data:?}");
                for byte in &byte_data {
                    print!("{:02X}", byte); // 以16进制格式打印，没有换行符
                }
                if byte_data[2] == 0x7B && byte_data[4] == 0xE0 {
                    println!("关机")
                } else if byte_data[2] == 0x6b && byte_data[4] == 0xE0 {
                    println!("左右扫风");
                } else if byte_data[0] == 0xb5 && byte_data[2] == 0xf5 {
                    println!("其他");
                } else {
                    //风速
                    let wind = (byte_data[2] >> 5) & 0x7;
                    //模式
                    let mode = (byte_data[4] >> 2) & 0x3;
                    // 温度
                    println!("温度原始数据{} ", byte_data[4]);
                    let temp = (byte_data[4] >> 4) & 0xf;
                    println!("温度：{temp} ");
                    //温度
                    match Speed::as_u8(wind) {
                        Speed::Auto => {
                            print!("自动,");
                        }
                        Speed::Low => {
                            print!("低风,");
                        }
                        Speed::Middle => {
                            print!("中风,");
                        }
                        Speed::High => {
                            print!("高风,");
                        }
                        Speed::Fixed => {
                            print!("固定风,");
                        }
                    }

                    match Mode::as_u8(mode) {
                        Mode::Cool => {
                            print!("制冷,");
                        }
                        Mode::Humidify => {
                            print!("抽湿,");
                        }
                        Mode::Heat => {
                            print!("制热,");
                        }
                        Mode::Auto => {
                            print!("自动,");
                        }
                    }
                    match R05D_Temp::as_u8(temp) {
                        R05D_Temp::T16 => {
                            print!("温度：16°C");
                        }
                        R05D_Temp::T17 => {
                            print!("温度：17°C");
                        }
                        R05D_Temp::T18 => {
                            print!("温度：18°C");
                        }
                        R05D_Temp::T19 => {
                            print!("温度：19°C");
                        }
                        R05D_Temp::T20 => {
                            print!("温度：20°C");
                        }
                        R05D_Temp::T21 => {
                            print!("温度：21°C");
                        }
                        R05D_Temp::T22 => {
                            print!("温度：22°C");
                        }
                        R05D_Temp::T23 => {
                            print!("温度：23°C");
                        }
                        R05D_Temp::T24 => {
                            print!("温度：24°C");
                        }
                        R05D_Temp::T25 => {
                            print!("温度：25°C");
                        }
                        R05D_Temp::T26 => {
                            print!("温度：26°C");
                        }
                        R05D_Temp::T27 => {
                            print!("温度：27°C");
                        }
                        R05D_Temp::T28 => {
                            print!("温度：28°C");
                        }
                        R05D_Temp::T29 => {
                            print!("温度：29°C");
                        }
                        R05D_Temp::T30 => {
                            print!("温度：30°C");
                        }
                        R05D_Temp::T31 => {
                            print!("温度：31°C");
                        }
                    }
                }

                println!(); // 打印完成后换行
            }
            FreeRtos::delay_ms(500);
        });

    let _ = std::thread::spawn(move || loop {
        info!("Starting RMT send");
        FreeRtos::delay_ms(1000);
        // 关机
        send_wave_code(&mut tx, 0xB2, 0xBF, 0xE0).unwrap();
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

// // h200_l200
// fn send_h200_l200(tx: &mut TxRmtDriver) -> Result<()> {
//     let p1 = Pulse::new(PinState::High, PulseTicks::new(2000).unwrap());
//     let p2 = Pulse::new(PinState::Low, PulseTicks::new(2000).unwrap());
//     let mut s = FixedLengthSignal::<1>::new();
//     s.set(0, &(p1, p2)).unwrap();
//     tx.start(s).unwrap();
//     Ok(())
// }

// // h200_l400
// fn send_h200_l400(tx: &mut TxRmtDriver) -> Result<()> {
//     let p1 = Pulse::new(PinState::High, PulseTicks::new(6000).unwrap());
//     let p2 = Pulse::new(PinState::Low, PulseTicks::new(4000).unwrap());
//     let mut s = FixedLengthSignal::<1>::new();
//     s.set(0, &(p1, p2)).unwrap();
//     tx.start(s).unwrap();
//     Ok(())
// }

// // h400_l200
// fn send_h400_l200(tx: &mut TxRmtDriver) -> Result<()> {
//     let p1 = Pulse::new(PinState::High, PulseTicks::new(4000).unwrap());
//     let p2 = Pulse::new(PinState::Low, PulseTicks::new(2000).unwrap());
//     let mut s = FixedLengthSignal::<1>::new();
//     s.set(0, &(p1, p2)).unwrap();
//     tx.start(s).unwrap();
//     Ok(())
// }
// 226440

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
    let p1 = Pulse::new(PinState::High, PulseTicks::new(540).unwrap());
    let p2 = Pulse::new(PinState::Low, PulseTicks::new(540).unwrap());
    let mut s = FixedLengthSignal::<1>::new();
    s.set(0, &(p1, p2)).unwrap();
    tx.start(s).unwrap();
    Ok(())
}

// 数据码1
fn send_1_code(tx: &mut TxRmtDriver) -> Result<()> {
    let p1 = Pulse::new(PinState::High, PulseTicks::new(540).unwrap());
    let p2 = Pulse::new(PinState::Low, PulseTicks::new(1620).unwrap());
    let mut s = FixedLengthSignal::<1>::new();
    s.set(0, &(p1, p2)).unwrap();
    tx.start(s).unwrap();
    Ok(())
}

// 分隔码
fn send_stop_code(tx: &mut TxRmtDriver) -> Result<()> {
    let p1 = Pulse::new(PinState::High, PulseTicks::new(540).unwrap());
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
    // 固定风
    Fixed = 0,
}

impl Speed {
    fn as_u8(data: u8) -> Speed {
        match data {
            5 => Speed::Auto,
            4 => Speed::Low,
            2 => Speed::Middle,
            1 => Speed::High,
            0 => Speed::Fixed,
            _ => panic!(),
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
        match data {
            2 => Mode::Auto,
            0 => Mode::Cool,
            1 => Mode::Humidify,
            3 => Mode::Heat,
            _ => panic!(),
        }
    }
}

// 温度
enum R05D_Temp {
    T16 = 15, //C(4,7)=1111,16℃
    T17 = 0,  //C(4,7)=0000,17℃
    T18 = 1,  //C(4,7)=0001,18℃
    T19 = 3,  //C(4,7)=0011,19℃
    T20 = 2,  //C(4,7)=0010,20℃
    T21 = 6,  //C(4,7)=0110,21℃
    T22 = 7,  //C(4,7)=0111,22℃
    T23 = 5,  //C(4,7)=0101,23℃
    T24 = 4,  //C(4,7)=0100,24℃
    T25 = 12, //C(4,7)=1100,25℃
    T26 = 13, //C(4,7)=1101,26℃
    T27 = 9,  //C(4,7)=1001,27℃
    T28 = 8,  //C(4,7)=1000,28℃
    T29 = 10, //C(4,7)=1010,29℃
    T30 = 11, //C(4,7)=1011,30℃ B24DBF4000FFB24DBF4000FFD5660010004B自动
    T31 = 14, //C(4,7)=1110,无定义，送风模式下使用 B24DBF4000FFB24DBF4000FFD5660000003B自动
}

impl R05D_Temp {
    fn as_u8(data: u8) -> R05D_Temp {
        match data {
            0 => R05D_Temp::T17,
            1 => R05D_Temp::T18,
            3 => R05D_Temp::T19,
            2 => R05D_Temp::T20,
            6 => R05D_Temp::T21,
            7 => R05D_Temp::T22,
            5 => R05D_Temp::T23,
            4 => R05D_Temp::T24,
            12 => R05D_Temp::T25,
            13 => R05D_Temp::T26,
            9 => R05D_Temp::T27,
            8 => R05D_Temp::T28,
            10 => R05D_Temp::T29,
            11 => R05D_Temp::T30,
            15 => R05D_Temp::T16,
            14 => R05D_Temp::T31,
            _ => panic!(),
        }
    }
}
