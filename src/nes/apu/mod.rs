
struct PulseChannel {
    duty_cycle: u8,
    length_counter_halt: u8,
    envelope: u8,
    sweep: u8,
    length_counter: u16,
}

impl PulseChannel {
    pub fn new() -> Self {
        PulseChannel {
            duty_cycle: 0x00,
            length_counter_halt: 0x00,
            envelope: 0x00,
            sweep: 0x00,
            length_counter: 0x0000,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
    }
}

struct NoiseChannel {
}

impl NoiseChannel {
    pub fn new() -> Self {
        NoiseChannel {
        }
    }
}

struct TriangleChannel {
}

impl TriangleChannel {
    pub fn new() -> Self {
        TriangleChannel {
        }
    }
}

pub struct Apu {
    pulse_channels: Vec<PulseChannel>,
    noise_channel: NoiseChannel,
    triangle_channel: TriangleChannel,
    cycle: u16, // 11-bit timer
}

impl Apu {
    pub fn new() -> Self {
        let mut pulse_channels = vec![];
        for _ in 0..2 {
            pulse_channels.push(PulseChannel::new());
        }

        Apu {
            pulse_channels: pulse_channels,
            noise_channel: NoiseChannel::new(),
            triangle_channel: TriangleChannel::new(),
            cycle: 0x0000,
        }
    }

    pub fn tick(&mut self) {
    }

    pub fn render_sound(&self, sound: &[u8]) {
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        info!("APU::write({:x}, {:x})", addr, data);
        match addr {
            0x4000...0x4003 => {
                self.pulse_channels[0].write(0x0003 & addr, data)
            },
            0x4004...0x4007 => {
                self.pulse_channels[1].write(0x0003 & addr, data)
            },
            0x4008...0x4013 => {
                // not implemented
            },
            _ => {
                panic!("invalid address:{:x} = {:x}", addr, data)
            },
        }
    }
}

