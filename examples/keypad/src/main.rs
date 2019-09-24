#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::clocks::Clocks;
use crate::hal::gpio::*;
use crate::hal::mailbox::Mailbox;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::{Bps, Duration, Instant};
use core::fmt::Write;
use keypad::{keypad_new, keypad_struct};

raspi3_boot::entry!(kernel_entry);
fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();
    let gp = gpio.split();
    let sys_timer = SysTimer::new();
    let sys_counter = sys_timer.split().sys_counter;

    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    let kp_r0 = gp.p5.into_pull_up_input();
    let kp_r1 = gp.p6.into_pull_up_input();
    let kp_r2 = gp.p13.into_pull_up_input();
    let kp_r3 = gp.p19.into_pull_up_input();

    let kp_c0 = gp.p17.into_push_pull_output();
    let kp_c1 = gp.p27.into_push_pull_output();
    let kp_c2 = gp.p22.into_push_pull_output();

    let kp = keypad_new!(PhoneKeypad {
        rows: (kp_r0, kp_r1, kp_r2, kp_r3,),
        columns: (kp_c0, kp_c1, kp_c2,),
    });

    let states = [
        [KeyState::new('1'), KeyState::new('2'), KeyState::new('3')],
        [KeyState::new('4'), KeyState::new('5'), KeyState::new('6')],
        [KeyState::new('7'), KeyState::new('8'), KeyState::new('9')],
        [KeyState::new('*'), KeyState::new('0'), KeyState::new('#')],
    ];

    let mut keypad = Keypad::new(states, kp);

    writeln!(serial, "Keypad example").ok();

    loop {
        let instant = sys_counter.get_time();
        if let Some(event) = keypad.get(&instant) {
            writeln!(serial, "{:?}", event).ok();
        }
    }
}

pub struct KeyState {
    key: char,
    state: bool,
    pub prev_pressed: bool,
    last_db: Instant,
}

impl KeyState {
    pub const fn new(key: char) -> Self {
        KeyState {
            key,
            state: false,
            prev_pressed: false,
            last_db: Instant { millis: 0 },
        }
    }

    pub fn key(&self) -> char {
        self.key
    }

    // (pressed, prev_pressed)
    pub fn pressed(&mut self, time: &Instant) -> (bool, bool) {
        let prev = self.prev_pressed;
        self.prev_pressed = if (*time - self.last_db) < DEBOUNCE_DURATION {
            false
        } else {
            self.state
        };
        (self.prev_pressed, prev)
    }

    pub fn long_pressed(&mut self, time: &Instant) -> bool {
        if (*time - self.last_db) < LONGPRESS_DURATION {
            false
        } else {
            true
        }
    }

    /// Returns true if state changed
    pub fn set(&mut self, time: &Instant, state: bool) -> bool {
        let prev = self.state;
        self.state = state;
        let changed = prev != self.state;
        if changed && state {
            self.last_db = *time;
        }
        changed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeypadEvent {
    KeyPress(char),
    LongPress(char),
}

struct Keypad {
    states: KeyStateMatrix,
    kp: PhoneKeypad,
}

keypad_struct! {
    struct PhoneKeypad {
        rows: (
          Pin5<Input<PullUp>>,
          Pin6<Input<PullUp>>,
          Pin13<Input<PullUp>>,
          Pin19<Input<PullUp>>,
        ),
        columns: (
            Pin17<Output<PushPull>>,
            Pin27<Output<PushPull>>,
            Pin22<Output<PushPull>>,
        ),
    }
}

const DEBOUNCE_DURATION: Duration = Duration::from_millis(25);
const LONGPRESS_DURATION: Duration = Duration::from_secs(1);

type KeyStateMatrix = [[KeyState; 3]; 4];

impl Keypad {
    pub fn new(states: KeyStateMatrix, kp: PhoneKeypad) -> Self {
        Keypad { states, kp }
    }

    pub fn get(&mut self, time: &Instant) -> Option<KeypadEvent> {
        let keys = self.kp.decompose();
        for (row_index, row) in keys.iter().enumerate() {
            for (col_index, key) in row.iter().enumerate() {
                let pressed = key.is_low().unwrap();
                let changed = self.states[row_index][col_index].set(time, pressed);
                let (_db_pressed, prev_pressed) = self.states[row_index][col_index].pressed(time);
                let long_pressed = self.states[row_index][col_index].long_pressed(time);
                if changed && prev_pressed {
                    let c = self.states[row_index][col_index].key();
                    return Some(match long_pressed {
                        false => KeypadEvent::KeyPress(c),
                        true => KeypadEvent::LongPress(c),
                    });
                }
            }
        }

        None
    }
}
