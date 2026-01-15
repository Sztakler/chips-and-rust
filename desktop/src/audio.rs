use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

struct SquareWave {
    phase_increment: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = { self.phase + self.phase_increment } % 1.0;
        }
    }
}

pub struct AudioSystem {
    device: AudioDevice<SquareWave>,
}

impl AudioSystem {
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, String> {
        let audio_subsystem = sdl_context.audio()?;

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| SquareWave {
            phase_increment: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.1,
        })?;

        Ok(AudioSystem { device })
    }

    pub fn sync(&self, sound_timer: u8) {
        if sound_timer > 0 {
            self.device.resume();
        } else {
            self.device.pause();
        }
    }
}
