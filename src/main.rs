//! Kerststal - 5 LED Candle Flicker
//!
//! This program makes 5 LEDs blink like candles using PWM fade effects.
//! GPIO pins: 0, 1, 2, 3, 4 (connected to LEDC channels 0-4)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::rng::Rng;
use esp_hal::{
    delay::Delay,
    clock::CpuClock,
    gpio::{AnyPin, DriveMode, Pin},
    ledc::{
        channel::{self, ChannelIFace, Number},
        timer::{self, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    time::Rate,
};


fn random_data(rng: &mut Rng) -> (u8, u16) {
    let coin_flip = rng.random() % 100;

    let (target_duty, speed_ms) = if coin_flip < 5 {
        (rng.random() % 10 + 5, rng.random() % 50 + 30)
    } else if coin_flip < 25 {
        (rng.random() % 40 + 40, rng.random() % 100 + 50)
    } else  {
        (rng.random() % 30 + 70, rng.random() % 200 + 150)
    };

    let duty_5bit = (target_duty * 31 / 100) as u8;
    (duty_5bit, speed_ms as u16)
}

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    
    let _systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    // Configure PWM timer at 24 kHz with 5-bit resolution
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let _ = lstimer0.configure(timer::config::Config {
        duty: timer::config::Duty::Duty5Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(24),
    });

    let mut pins: [Option<AnyPin>; 5] = [
        Some(peripherals.GPIO0.degrade()),
        Some(peripherals.GPIO1.degrade()),
        Some(peripherals.GPIO2.degrade()),
        Some(peripherals.GPIO3.degrade()),
        Some(peripherals.GPIO4.degrade()),
    ];

    let channel_numbers = [
        Number::Channel0,
        Number::Channel1,
        Number::Channel2, 
        Number::Channel3,
        Number::Channel4,
    ];

    let channels: [_; 5] = core::array::from_fn(|i| {
        let pin = pins[i].take().unwrap(); 
        
        let mut chan = ledc.channel(channel_numbers[i], pin);
        chan.configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 100,
            drive_mode: DriveMode::PushPull,
        }).unwrap();
        chan
    });
    let mut rng = Rng::new();

    let delay = Delay::new();

    loop {
        
        let mut rand_data = [(0u8, 0u16); 5];
        // Generate random minimum brightness for each LED
        for ii in 0..5 {
            rand_data[ii] = random_data(&mut rng);
        }

        // Fade back down
        for ii in 0..5 {
             let (min, speed) = rand_data[ii];
             let _ = channels[ii].start_duty_fade(100, min, speed);
             delay.delay_millis(rng.random() % 50);
        }
        
        delay.delay_millis(rand_data.iter().map(|(_, speed)| *speed).max().unwrap() as u32);

        // Fade up again
        for ii in 0..5 {
            let (min, speed) = rand_data[ii];
            let _ = channels[ii].start_duty_fade(min, 100, speed);
            delay.delay_millis(rng.random() % 50);
        }
        
        delay.delay_millis(rand_data.iter().map(|(_, speed)| *speed).max().unwrap() as u32);

        if rng.random() % 8 == 0 {
            for ii in 2..=4 {
                let _ = channels[ii].start_duty_fade(100, 5, 150);
            }
            delay.delay_millis(150);
            for ii in 2..=4 {
                let _ = channels[ii].start_duty_fade(5, 100, 150);
            }
            delay.delay_millis(150);
        }

        let pause_time = 500 + (rng.random() % 1501);
        delay.delay_millis(pause_time);
    }
}
