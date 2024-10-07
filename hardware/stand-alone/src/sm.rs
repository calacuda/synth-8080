use esp_idf_svc::hal::adc::config::Config;
// use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::gpio::*;
use lib::{Float, OscType, communication::command::SynthCmd};
use std::default::Default;

#[derive(Debug, Clone, Copy)]
struct LFO {
    pub vol: Float,
    pub speed: Float,
    pub osc_type: OscType
}

impl Default for LFO {
    fn default() -> Self {
        LFO {
            vol: 0.0,
            speed: 0.0,
            osc_type: OscType::Sine,
        }
    }
}


#[derive(Debug, Default)]
struct ADBDR {
    pub attack: Float,
    pub decay_1: Float,
    pub threshold: Float,
    pub decay_2: Float,
}

#[derive(Debug, Default)]
struct ADSR {
    pub attack: Float,
    pub decay: Float,
    pub sustain: Float,
}

#[derive(Debug, Default)]
struct AD {
    pub attack: Float,
    pub decay: Float,
}

#[derive(Debug, Default)]
struct Echo {
    pub speed: Float,
    pub decay: Float,
}

#[derive(Debug, Default)]
struct Chorus {
    pub speed: Float,
    pub decay: Float,
}

#[derive(Debug, Default)]
struct Reverb {
    pub gain: Float,
    pub decay: Float,
}

#[derive(Debug)]
struct LEDs {
    src_mod: u8,
    src_out: u8,
    dest_mod: u8,
    dest_in: u8,
    lfo_wf: [OscType; 4],
    vco: OscType,
    filter: [bool; 3],
}

impl Default for LEDs {
    fn default() -> Self {
        Self {
            lfo_wf: [OscType::Sine, OscType::Sine, OscType::Sine, OscType::Sine],
            vco: OscType::Sine,
            filter: [true, false, false],
            ..Default::default()
        }
    }
}

struct Switches<'a> {
    rows: [PinDriver<'a, AnyInputPin, Input>; 8],
    cols: [PinDriver<'a, AnyOutputPin, Output>; 3],
    connect: PinDriver<'a, AnyInputPin, Input>,
}

// macro_rules! adc_read {
//     () => {
//         // The macro will expand into the contents of this block.
//         println!("Hello!")
//     };
// }

struct ControlsState<'a> {
    // pins: Pins,
    adc: AdcDriver<'a, ADC1>,
    switch_1: (AdcChannelDriver<'a, { attenuation::DB_11 }, Gpio1>, [PinDriver<'a, AnyOutputPin, Output>; 4]),
    switch_2: (AdcChannelDriver<'a, { attenuation::DB_11 }, Gpio2>, [PinDriver<'a, AnyOutputPin, Output>; 4]),
    lfo_s: [LFO; 4],
    vol: Float,
    gain: Float,
    adbdr: ADBDR,
    adsr: ADSR,
    ad: AD,
    echo: Echo,
    chorus: Chorus,
    reverb: Reverb,
    leds: LEDs,
    switches: Switches<'a>,
}

impl ControlsState<'_> {
    pub fn new (pins: Pins, adc1: ADC1) -> anyhow::Result<Self> {
        let config = Config::new().calibration(true);
        let mut adc = AdcDriver::new(adc1, &config)?;


        Ok(Self {
            // pins,
            adc,
            // switch_1: (AdcChannelDriver::<{ attenuation::DB_11 }, _>::new(pins.gpio1)?, [PinDriver::output(pins.gpio38.into())?, PinDriver::output(pins.gpio37.into()), PinDriver::output(pins.gpio36.into()), PinDriver::output(pins.gpio35.into())]),
            switch_1: (AdcChannelDriver::<{ attenuation::DB_11 }, _>::new(pins.gpio1)?, [PinDriver::output(AnyOutputPin::from(pins.gpio38))?, PinDriver::output(AnyOutputPin::from(pins.gpio37))?, PinDriver::output(AnyOutputPin::from(pins.gpio36))?, PinDriver::output(AnyOutputPin::from(pins.gpio35))?]),
            // switch_2: (AdcChannelDriver::<{ attenuation::DB_11 }, _>::new(pins.gpio2)?, [PinDriver::output(pins.gpio4.into())?, PinDriver::output(pins.gpio5.into()), PinDriver::output(pins.gpio6.into()), PinDriver::output(pins.gpio7.into())]),
            switch_2: (AdcChannelDriver::<{ attenuation::DB_11 }, _>::new(pins.gpio2)?, [PinDriver::output(AnyOutputPin::from(pins.gpio4))?, PinDriver::output(AnyOutputPin::from(pins.gpio5))?, PinDriver::output(AnyOutputPin::from(pins.gpio6))?, PinDriver::output(AnyOutputPin::from(pins.gpio7))?]),
            lfo_s: [LFO::default(); 4],
            vol: 0.0,
            gain: 0.0,
            adbdr: ADBDR::default(),
            adsr: ADSR::default(),
            ad: AD::default(),
            echo: Echo::default(),
            chorus: Chorus::default(),
            reverb: Reverb::default(),
            leds: LEDs::default(),
            switches: Switches {
                rows: [
                    PinDriver::input(AnyInputPin::from(pins.gpio16))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio17))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio18))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio8))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio3))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio46))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio9))?,
                    PinDriver::input(AnyInputPin::from(pins.gpio10))?,
                ],
                cols: [
                    PinDriver::output(AnyOutputPin::from(pins.gpio11))?,
                    PinDriver::output(AnyOutputPin::from(pins.gpio12))?,
                    PinDriver::output(AnyOutputPin::from(pins.gpio13))?,
                ],
                connect: PinDriver::input(AnyInputPin::from(pins.gpio21))?,
            },
        })
    }

    pub fn state(&mut self) -> anyhow::Result<Vec<SynthCmd>> {
        let mut cmds = Vec::new();
        
        // gen_cmd(self, &mut cmds, );
        
        // volume
        let new_vol = self.read_pot(true, 8)?;
        if self.vol != new_vol {
            self.vol = new_vol;
            cmds.push(SynthCmd::VcoVol(new_vol));
        }

        // AD decay
        let new_ad_atk = self.read_pot(false, 0)?;
        if self.ad.attack != new_ad_atk {
            self.ad.attack = new_ad_atk;
            cmds.push(SynthCmd::AdAtk(new_ad_atk));
        }

        // AD decay
        let new_ad_decay = self.read_pot(false, 1)?;
        if self.ad.decay != new_ad_decay {
            self.ad.decay = new_ad_decay;
            cmds.push(SynthCmd::AdDecay(new_ad_decay));
        }

        // overdrive gain
        let new_gain = self.read_pot(false, 2)?;
        if self.gain != new_gain {
            self.gain = new_gain;
            cmds.push(SynthCmd::OdGain(new_gain));
        }

        Ok(cmds)
    }

    pub fn read_pot(&mut self, mux_1: bool, pot_select: u8) -> anyhow::Result<Float> { 
        let mut pins = if mux_1 {
            &mut self.switch_1.1
        } else {
            &mut self.switch_2.1
        };

        for (i, pin) in pins.iter_mut().enumerate() {
            if (i as u8) & pot_select == i  as u8 {
                pin.set_high()?;
            } else {
                pin.set_low()?;
            }
        }

        Ok(if mux_1 {
            self.adc.read(&mut self.switch_1.0)? as Float / 4095.0
        } else {
            self.adc.read(&mut self.switch_2.0)? as Float / 4095.0
        })
    }
}

