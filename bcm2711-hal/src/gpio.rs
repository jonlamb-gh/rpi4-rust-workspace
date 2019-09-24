//! General Purpose Input / Output
//!
//! TODO - update this once bcm2711 docs are available
//! - add all the pins
//!
//! See the pinout:
//! https://pinout.xyz/

use bcm2711::gpio::*;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use cortex_a::asm;
use hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use void::Void;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;
pub struct AF4;
pub struct AF5;

pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

pub struct Parts {
    /// Pins
    pub p0: Pin0<Input<Floating>>,
    pub p1: Pin1<Input<Floating>>,
    pub p2: Pin2<Input<Floating>>,
    pub p3: Pin3<Input<Floating>>,
    pub p4: Pin4<Input<Floating>>,
    pub p5: Pin5<Input<Floating>>,
    pub p6: Pin6<Input<Floating>>,
    pub p7: Pin7<Input<Floating>>,
    pub p8: Pin8<Input<Floating>>,
    pub p9: Pin9<Input<Floating>>,
    pub p10: Pin10<Input<Floating>>,
    pub p11: Pin11<Input<Floating>>,
    pub p13: Pin13<Input<Floating>>,
    pub p14: Pin14<Input<Floating>>,
    pub p15: Pin15<Input<Floating>>,
    pub p17: Pin17<Input<Floating>>,
    pub p19: Pin19<Input<Floating>>,
    pub p20: Pin20<Input<Floating>>,
    pub p22: Pin22<Input<Floating>>,
    pub p27: Pin27<Input<Floating>>,
}

impl GpioExt for GPIO {
    type Parts = Parts;

    fn split(self) -> Parts {
        // Each pin gets a copy of the GPIO vaddr
        Parts {
            p0: Pin0 {
                pin: 0,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p1: Pin1 {
                pin: 1,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p2: Pin2 {
                pin: 2,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p3: Pin3 {
                pin: 3,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p4: Pin4 {
                pin: 4,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p5: Pin5 {
                pin: 5,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p6: Pin6 {
                pin: 6,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p7: Pin7 {
                pin: 7,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p8: Pin8 {
                pin: 8,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p9: Pin9 {
                pin: 9,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p10: Pin10 {
                pin: 10,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p11: Pin11 {
                pin: 11,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p13: Pin13 {
                pin: 13,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p14: Pin14 {
                pin: 14,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p15: Pin15 {
                pin: 15,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p17: Pin17 {
                pin: 17,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p19: Pin19 {
                pin: 19,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p20: Pin20 {
                pin: 20,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p22: Pin22 {
                pin: 22,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
            p27: Pin27 {
                pin: 27,
                gpio: GPIO::new(),
                _mode: PhantomData,
            },
        }
    }
}

/// GPIO pull-up/down clock sequence wait cycles
const WAIT_CYCLES: usize = 150;

macro_rules! gpio {
    (
        // struct field name, register field type name
        $GPFSELfn:ident, $GPFSELn:ident,
        $GPPUDCLKfn:ident, $GPPUDCLKx:ident,
        $GPLEVfn:ident, $GPLEVx:ident,
        $GPSETfn:ident, $GPSETx:ident,
        $GPCLRfn:ident, $GPCLRx:ident,
        [$($PXi:ident: ($pxi:ident, $FSELi:ident, $PUDCLKi:ident, $MODE:ty),)+]
    ) => {
$(
pub struct $PXi<MODE> {
    pin: u32,
    gpio: GPIO,
    _mode: PhantomData<MODE>,
}

impl<MODE> Deref for $PXi<MODE> {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        &self.gpio
    }
}

impl<MODE> DerefMut for $PXi<MODE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gpio
    }
}

impl<MODE> $PXi<MODE> {
    /// Configures the pin to operate in AF0 mode
    pub fn into_alternate_af0(mut self) -> $PXi<Alternate<AF0>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::AF0);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF1 mode
    pub fn into_alternate_af1(mut self) -> $PXi<Alternate<AF1>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::AF1);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF2 mode
    pub fn into_alternate_af2(mut self) -> $PXi<Alternate<AF2>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::AF2);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF3 mode
    pub fn into_alternate_af3(mut self) -> $PXi<Alternate<AF3>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::AF3);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF4 mode
    pub fn into_alternate_af4(mut self) -> $PXi<Alternate<AF4>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::AF4);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF5 mode
    pub fn into_alternate_af5(mut self) -> $PXi<Alternate<AF5>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::AF5);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate as a floating input pin
    pub fn into_floating_input(mut self) -> $PXi<Input<Floating>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::Input);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled down input pin
    pub fn into_pull_down_input(mut self) -> $PXi<Input<PullDown>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::Input);
        self.pud.modify(PullUpDown::PuD::PullDown);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_pull_up_input(mut self) -> $PXi<Input<PullUp>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::Input);
        self.pud.modify(PullUpDown::PuD::PullUp);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }

    /// Configures the pin to operate as an push pull output pin
    pub fn into_push_pull_output(mut self) -> $PXi<Output<PushPull>> {
        // Select function
        self.$GPFSELfn.modify($GPFSELn::$FSELi::Output);
        self.pud.modify(PullUpDown::PuD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.modify($GPPUDCLKx::$PUDCLKi::Set);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKfn.write(0);

        $PXi { pin: self.pin, gpio: self.gpio, _mode: PhantomData }
    }
}

impl<MODE> OutputPin for $PXi<Output<MODE>> {
    type Error = Void;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let pin = 1 << self.pin;
        Ok(self.$GPSETfn.write(pin))
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let pin = 1 << self.pin;
        Ok(self.$GPCLRfn.write(pin))
    }
}

impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|b| !b)
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.$GPLEVfn.read() & (1 << self.pin) == 0)
    }
}

impl<MODE> InputPin for $PXi<Input<MODE>> {
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|b| !b)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.$GPLEVfn.read() & (1 << self.pin) == 0)
    }
}
)+
    }
}

gpio!(
    fun_sel0,
    FunSel0,
    pud_clk0,
    PullUpDownClock0,
    level0,
    PinLevel0,
    set0,
    Set0,
    clr0,
    Clear0,
    [
        Pin0: (p0, Pin0, Pin0, Input<Floating>),
        Pin1: (p1, Pin1, Pin1, Input<Floating>),
        Pin2: (p2, Pin2, Pin2, Input<Floating>),
        Pin3: (p3, Pin3, Pin3, Input<Floating>),
        Pin4: (p4, Pin4, Pin4, Input<Floating>),
        Pin5: (p5, Pin5, Pin5, Input<Floating>),
        Pin6: (p6, Pin6, Pin6, Input<Floating>),
        Pin7: (p7, Pin7, Pin7, Input<Floating>),
        Pin8: (p8, Pin8, Pin8, Input<Floating>),
        Pin9: (p9, Pin9, Pin9, Input<Floating>),
    ]
);

gpio!(
    fun_sel1,
    FunSel1,
    pud_clk0,
    PullUpDownClock0,
    level0,
    PinLevel0,
    set0,
    Set0,
    clr0,
    Clear0,
    [
        Pin10: (p10, Pin10, Pin10, Input<Floating>),
        Pin11: (p11, Pin11, Pin11, Input<Floating>),
        Pin13: (p13, Pin13, Pin13, Input<Floating>),
        Pin14: (p14, Pin14, Pin14, Input<Floating>),
        Pin15: (p15, Pin15, Pin15, Input<Floating>),
        Pin17: (p17, Pin17, Pin17, Input<Floating>),
        Pin19: (p19, Pin19, Pin19, Input<Floating>),
    ]
);

gpio!(
    fun_sel2,
    FunSel2,
    pud_clk0,
    PullUpDownClock0,
    level0,
    PinLevel0,
    set0,
    Set0,
    clr0,
    Clear0,
    [
        Pin20: (p20, Pin20, Pin20, Input<Floating>),
        Pin22: (p22, Pin22, Pin22, Input<Floating>),
        Pin27: (p27, Pin27, Pin27, Input<Floating>),
    ]
);
