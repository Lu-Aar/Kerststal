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
    gpio::DriveMode,
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

    let led0 = peripherals.GPIO0;
    let led1 = peripherals.GPIO1;
    let led2 = peripherals.GPIO2;
    let led3 = peripherals.GPIO3;
    let led4 = peripherals.GPIO4;

    // Configure all 5 LEDC channels for concurrent PWM control
    let mut channel0 = ledc.channel(Number::Channel0, led0);
    let _ = channel0.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 100,
        drive_mode: DriveMode::PushPull
    });
    let mut channel1 = ledc.channel(Number::Channel1, led1);
    let _ = channel1.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 100,
        drive_mode: DriveMode::PushPull,
    });
    let mut channel2 = ledc.channel(Number::Channel2, led2);
    let _ = channel2.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 100,
        drive_mode: DriveMode::PushPull,
    });
    let mut channel3 = ledc.channel(Number::Channel3, led3);
    let _ = channel3.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 100,
        drive_mode: DriveMode::PushPull,
    });
    let mut channel4 = ledc.channel(Number::Channel4, led4);
    let _ = channel4.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 100,
        drive_mode: DriveMode::PushPull,
    });

    let mut rng = Rng::new();

    let delay = Delay::new();

    loop {
        
        // Generate random minimum brightness for each LED
        let (min0, rng_speed0) = random_data(&mut rng);
        let (min1, rng_speed1) = random_data(&mut rng);
        let (min2, rng_speed2) = random_data(&mut rng);
        let (min3, rng_speed3) = random_data(&mut rng);
        let (min4, rng_speed4) = random_data(&mut rng);

        // Fade back down
        let _ = channel0.start_duty_fade(100, min0, rng_speed0);
        delay.delay_millis(rng.random() % 50);
        let _ = channel1.start_duty_fade(100, min1, rng_speed1);
        delay.delay_millis(rng.random() % 50);
        let _ = channel2.start_duty_fade(100, min2, rng_speed2);
        delay.delay_millis(rng.random() % 50);
        let _ = channel3.start_duty_fade(100, min3, rng_speed3);
        delay.delay_millis(rng.random() % 50);
        let _ = channel4.start_duty_fade(100, min4, rng_speed4);
        
        delay.delay_millis(rng_speed0.max(rng_speed1).max(rng_speed2).max(rng_speed3).max(rng_speed4) as u32);

        // Start all fades simultaneously
        let _ = channel0.start_duty_fade(min0, 100, rng_speed0);
        delay.delay_millis(rng.random() % 50);
        let _ = channel1.start_duty_fade(min1, 100, rng_speed1);
        delay.delay_millis(rng.random() % 50);
        let _ = channel2.start_duty_fade(min2, 100, rng_speed2);
        delay.delay_millis(rng.random() % 50);
        let _ = channel3.start_duty_fade(min3, 100, rng_speed3);
        delay.delay_millis(rng.random() % 50);
        let _ = channel4.start_duty_fade(min4, 100, rng_speed4);
        
        delay.delay_millis(rng_speed0.max(rng_speed1).max(rng_speed2).max(rng_speed3).max(rng_speed4) as u32);

        if rng.random() % 8 == 0 {
            let _ = channel2.start_duty_fade(100, 5, 150);
            let _ = channel3.start_duty_fade(100, 5, 150);
            let _ = channel4.start_duty_fade(100, 5, 150);
            delay.delay_millis(150);
            let _ = channel2.start_duty_fade(5, 100, 150);
            let _ = channel3.start_duty_fade(5, 100, 150);
            let _ = channel4.start_duty_fade(5, 100, 150);
            delay.delay_millis(150);
        }

        let pause_time = 500 + (rng.random() % 1501);
        delay.delay_millis(pause_time);
    }
}
