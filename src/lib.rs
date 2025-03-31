use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

pub struct WirelessWireTx<T: OutputPin> {
    pin: T,
}

impl<T: OutputPin> WirelessWireTx<T> {
    pub fn new(pin: T) -> Self {
        Self { pin }
    }

    pub fn send(&mut self, data: &[u8], delay: &mut impl DelayNs) -> Result<(), T::Error> {
        // Send header.
        for _ in 0..3 {
            self.send_byte(0x00, delay)?;
        }

        // Send sync byte
        self.send_byte(0x55, delay)?;
        // Send payload length
        self.send_byte(data.len() as u8, delay)?;

        for &datum in data {
            self.send_byte(datum, delay)?;
        }

        Ok(())
    }

    pub fn send_byte(&mut self, byte: u8, delay: &mut impl DelayNs) -> Result<(), T::Error> {
        for i in 0..8 {
            let bit = (byte >> i) & 0x01 != 0;
            self.send_bit(bit, delay)?;
        }
        Ok(())
    }

    pub fn send_bit(&mut self, bit: bool, delay: &mut impl DelayNs) -> Result<(), T::Error> {
        self.pin.set_high()?;
        if bit {
            delay.delay_us(400);
        } else {
            delay.delay_us(800);
        }

        self.pin.set_low()?;
        delay.delay_us(600);
        Ok(())
    }
}

pub struct WirelessWireRx<T: InputPin> {
    pin: T,
}

impl<T: InputPin> WirelessWireRx<T> {
    pub fn new(pin: T) -> Self {
        Self { pin }
    }

    pub fn receive(&mut self, delay: &mut impl DelayNs, buffer: &mut [u8]) -> Option<u8> {
        self.wait_for_header(delay);
        if self.read_byte(delay)? != 0x55 {
            return None;
        }

        let len = self.read_byte(delay)? as usize;
        if len > buffer.len() {
            return None;
        }

        for i in 0..len {
            buffer[i] = self.read_byte(delay)?;
        }

        Some(len as u8)
    }

    pub fn read_byte(&mut self, delay: &mut impl DelayNs) -> Option<u8> {
        let mut byte = 0;
        for i in 0..8 {
            let bit = self.read_bit(delay)?;
            byte |= (bit as u8) << i;
        }
        Some(byte)
    }

    pub fn read_bit(&mut self, delay: &mut impl DelayNs) -> Option<bool> {
        let pulse = self.measure_pulse(delay).unwrap_or(0);

        if pulse < 100 {
            return None;
        }

        if (180..=500).contains(&pulse) {
            Some(true)
        } else if (501..=900).contains(&pulse) {
            Some(false)
        } else {
            None
        }
    }

    fn wait_for_header(&mut self, delay: &mut impl DelayNs) {
        let mut bit_count = 0;
        while bit_count < 24 {
            if !self.read_bit(delay).unwrap_or(true) {
                bit_count += 1;
            } else {
                bit_count = 0;
            }
        }
    }

    fn measure_pulse(&mut self, delay: &mut impl DelayNs) -> Result<u16, T::Error> {
        while self.pin.is_low()? {
            // Wait for the transmission to begin.
        }

        let mut duration = 0;
        while self.pin.is_high()? {
            delay.delay_us(10);
            duration += 10;

            if duration > 2000 {
                break;
            }
        }

        Ok(duration)
    }
}
