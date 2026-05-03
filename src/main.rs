//! Kerststal - 5 LED Candle Flicker
//!
//! This program makes 5 LEDs blink like candles using PWM fade effects.
//! GPIO pins: 0, 1, 2, 3, 4 (connected to LEDC channels 0-4)

#![no_std]
#![no_main]

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::rng::Rng;
use esp_hal::{
    clock::CpuClock,
    gpio::DriveMode,
    ledc::{
        channel::{self, ChannelIFace, Number},
        timer::{self, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    time::Rate,
};
use esp_println::println;

// Busy-wait delay that doesn't need time driver
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 10_000) {
        core::hint::spin_loop();
    }
}

/// Perform one blink cycle: fade from minimum to 100% and back
async fn blink<'a, S: timer::TimerSpeed + 'a>(
    channel: &mut impl ChannelIFace<'a, S>,
    minimum: u8,
    duration_ms: u16,
    should_dip: bool,
) {
    let _ = channel.start_duty_fade(minimum, 100, duration_ms);
    Timer::after(Duration::from_millis(duration_ms as u64)).await;
    let _ = channel.start_duty_fade(100, minimum, duration_ms);
    Timer::after(Duration::from_millis(duration_ms as u64)).await;

    if should_dip {
        Timer::after(Duration::from_millis(50)).await;
        let _ = channel.start_duty_fade(100, 20, 100);
        Timer::after(Duration::from_millis(100)).await;
        let _ = channel.start_duty_fade(20, 100, 100);
        Timer::after(Duration::from_millis(100)).await;
    }
}

fn random_data(rng: &mut Rng) -> (u8, u16, bool) {
    let min = 5 + (rng.random() % 36) as u8;

    let rng_speed = if rng.random() % 4 == 0 {
        300 + (rng.random() % 301) as u16
    } else {
        150 + (rng.random() % 151) as u16
    };
    let dip = rng.random() % 5 == 0;
    (min, rng_speed, dip)
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

    loop {
        // Generate random minimum brightness for each LED
        let (min0, rng_speed0, dip0) = random_data(&mut rng);
        let (min1, rng_speed1, dip1) = random_data(&mut rng);
        let (min2, rng_speed2, dip2) = random_data(&mut rng);
        let (min3, rng_speed3, dip3) = random_data(&mut rng);
        let (min4, rng_speed4, dip4) = random_data(&mut rng);

        futures::join!(
            blink(&mut channel0, min0, rng_speed0, dip0),
            blink(&mut channel1, min1, rng_speed1, dip1),
            blink(&mut channel2, min2, rng_speed2, dip2),
            blink(&mut channel3, min3, rng_speed3, dip3),
            blink(&mut channel4, min4, rng_speed4, dip4),
        );

        // Occasionally have all LEDs do a quick simultaneous dip for extra flicker effect
        if rng.random() % 3 == 0 {
            futures::join!(
                blink(&mut channel2, 5, 80, true),
                blink(&mut channel3, 5, 80, true),
                blink(&mut channel4, 5, 80, true),
            );
        }

        println!("Cycle complete!");
        let pause_time = 500 + (rng.random() % 1501) as u64;
        Timer::after(Duration::from_millis(pause_time)).await;
    }
}
