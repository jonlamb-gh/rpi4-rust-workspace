# rpi4-rust-workspace

Rust workspace for RPi4 bare metal things

A lot of the things in here were inspired by [rust-raspi3-OS-tutorials](https://github.com/rust-embedded/rust-raspi3-OS-tutorials).

## Crates

* [bcm2711](bcm2711/) : Device crate, registers defined via [bounded-registers](https://github.com/auxoncorp/bounded-registers)
* [bcm2711-hal](bcm2711-hal/) : [embedded-hal](https://github.com/rust-embedded/embedded-hal) trait impls
* [display](display/) : Double buffered DMA graphics/display library
* [raspi3_boot](raspi3_boot/) : Mostly copied from [rust-raspi3-OS-tutorials](https://github.com/rust-embedded/rust-raspi3-OS-tutorials)

## Examples

* [analog-clock](examples/analog-clock/src/main.rs) : Port of the [embedded-graphics](https://github.com/jamwaffles/embedded-graphics) `analog-clock` example
* [embedded-graphics](examples/embedded-graphics/src/main.rs) : Simple [embedded-graphics](https://github.com/jamwaffles/embedded-graphics) example
* [eth](examples/eth/src/main.rs) : On-board GENET Ethernet example
* [ip](examples/ip/src/main.rs) : [smoltcp](https://github.com/smoltcp-rs/smoltcp) IP stack / TCP server example
* [mbox](examples/mbox/src/main.rs) : Reads various things using the [Mailbox property interface](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface)
* [mem2mem-dma](examples/mem2mem-dma/src/main.rs) : Simple DMA transfer example
* [uart1](examples/uart1/src/main.rs) : UART1 example

## Building

```rust
cargo xbuild
```

Copy elf to binary:

```bash
cargo objcopy -- -O binary target/$(TARGET)/release/img /tmp/img.bin
```

## Simulating

[Comming soon?](https://lists.gnu.org/archive/html/qemu-devel/2019-09/msg00681.html)

https://gitlab.com/philmd/qemu/commits/raspi4_wip

```bash
# For output on UART1
qemu-system-aarch64 -M raspi4 -nographic -serial null -serial mon:stdio -kernel /path/to/binary
```

TODO - cusomer runner / `cargo xrun` support:

```toml
[target.'cfg(target_os = "none")']
runner = "tools/qemu-runner"
```

## U-boot

Using 64 bit U-boot:

```bash
git clone --depth 1 https://github.com/u-boot/u-boot.git u-boot
ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- make rpi_4_defconfig
ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- make
```

```bash
U-Boot 2020.04-rc2-00048-gf2a73d6867 (Feb 16 2020 - 08:29:41 -0800)

aarch64-linux-gnu-gcc (Ubuntu/Linaro 8.3.0-6ubuntu1) 8.3.0
GNU ld (GNU Binutils for Ubuntu) 2.32
```

Environment:

```bash
setenv imgname img.bin

# Normally for bare metal
#setenv loadaddr 0x80000

# Put it somewhere else, so we don't overwrite u-boot
setenv loadaddr 0x0100000

# Disable data cache because u-boot turns it on and my stuff isn't ready for it
setenv bootimg 'tftp ${loadaddr} ${serverip}:${imgname}; dcache flush; dcache off; go ${loadaddr}'
```

## SD Card

Files:

```bash
/card
├── config.txt
├── fixup4.dat
├── start4.elf
├── u-boot.bin
└── uboot.env
```

Contents of `config.txt`:

```bash
enable_uart=1
arm_64bit=1
kernel=u-boot.bin
```

## Links

- [BCM2711](https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2711/README.md)
- [RPi4 datasheet](https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2711/rpi_DATA_2711_1p0_preliminary.pdf)
- [Revised BCM2837 doc](https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf)
- [Bare metal boot code for ARMv8-A](http://infocenter.arm.com/help/topic/com.arm.doc.dai0527a/DAI0527A_baremetal_boot_code_for_ARMv8_A_processors.pdf)
- [Bare-metal C++ code](https://github.com/rsta2/circle)
- [bcm2711-rpi-4-b.dtb](https://github.com/Hexxeh/rpi-firmware/blob/master/bcm2711-rpi-4-b.dtb)
- [Linux GENET drivers](https://github.com/torvalds/linux/tree/master/drivers/net/ethernet/broadcom/genet)
- [U-boot GENET driver](https://github.com/u-boot/u-boot/blob/master/drivers/net/bcmgenet.c)
- [More info on RPi4](https://www.raspberrypi.org/forums/viewtopic.php?t=244479&start=25)
- [U-boot load over serial with kermit](http://blog.mezeske.com/?p=483)
- [Inside the RPi4 article, has some Eth specs/etc](https://cdn.shopify.com/s/files/1/1560/1473/files/Inside_Raspberry_Pi_4.pdf)
- [My RPi3 workspace](https://github.com/jonlamb-gh/rpi3-rust-workspace)
