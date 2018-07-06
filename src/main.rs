extern crate hidapi;
use hidapi::{HidResult, HidDevice};

struct Keyboard<'a> {
    device: HidDevice<'a>,
    sequence: u8,
}

impl<'a> Keyboard<'a> {
    fn new(context: &'a hidapi::HidApi) -> HidResult<Self> {
        let vendor_id = 0x24f0;
        let product_id = 0x2020;
        let interface_number = 2;
        for device in context.devices().into_iter() {
            if device.vendor_id == vendor_id && device.product_id == product_id && device.interface_number == interface_number {
                let device = context.open_path(&device.path)?;
                return Ok(Self {
                    device: device,
                    sequence: 0,
                });
            }
        }
        Err("not found")
    }

    fn feature_reports(&mut self, data: &[u8]) -> HidResult<()> {
        let buff = &mut [0u8; 65];
        buff[1..1 + data.len()].copy_from_slice(data);
        buff[3] = self.sequence;
        self.device.send_feature_report(buff)?;
        let next_sequence = buff[3].overflowing_add(1).0;
        buff[2] = 0;
        buff[3] = next_sequence;
        let n = self.device.get_feature_report(buff)?;
        assert!(n > 4, "Expected at least 4 bytes read, got {}", n);
        assert_eq!(buff[2], 0x14, "Expected buff[2] == 0x14, got 0x{:02x}", buff[2]);
        assert_eq!(buff[3], self.sequence, "Wrong sequence number in response: got {}, expected {}", buff[3], self.sequence);
        self.sequence = next_sequence;
        Ok(())
    }

    fn initialize(&mut self) -> HidResult<()> {
        let data = &[0x00, 0x13, 0x00, 0x4d, 0x43, 0x49, 0x51, 0x46, 0x49, 0x46, 0x45, 0x44, 0x4c,
        0x48, 0x39, 0x46, 0x34, 0x41, 0x45, 0x43, 0x58, 0x39, 0x31, 0x36, 0x50, 0x42, 0x44, 0x35,
        0x50, 0x33, 0x41, 0x33, 0x30, 0x37, 0x38];
        self.feature_reports(data)
    }

    fn trigger(&mut self) -> HidResult<()> {
        let data = &[0, 45, 0, 15, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF];
        self.feature_reports(data)
    }

    fn firmware(&mut self) -> HidResult<String> {
        let data = &[0x00, 0x11, 0x06, 0x4d, 0x43, 0x49, 0x51, 0x46, 0x49, 0x46, 0x45, 0x44, 0x4c,
        0x48, 0x39, 0x46, 0x34, 0x41, 0x45, 0x43, 0x58, 0x39, 0x31, 0x36, 0x50, 0x42, 0x44, 0x35,
        0x50, 0x33, 0x41, 0x33, 0x30, 0x37, 0x38];
        self.feature_reports(data)?;
        let buff = &mut [0u8; 65];
        let n = self.device.get_feature_report(buff)?;
        assert!(n > 8, "Expected at least 8 bytes read, got {}", n);
        Ok(format!("{}.{}.{}.{}", buff[4], buff[5], buff[6], buff[7]))
    }
}

fn main() {
    let context = hidapi::HidApi::new().unwrap();
    let mut keyboard = match Keyboard::new(&context) {
        Ok(keyboard) => keyboard,
        Err(e) => {
            println!("Failed to open keyboard: {}", e);
            return;
        },
    };
    keyboard.initialize().unwrap();
    println!("Firmware version: {}", keyboard.firmware().unwrap());
    keyboard.trigger().unwrap();
}
