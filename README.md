# rpi4-rust-workspace

Rust workspace for RPi4 bare metal things

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

## U-boot

Using 64 bit U-boot:

```bash
git clone --depth 1 https://github.com/u-boot/u-boot.git u-boot
ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- make rpi_4_defconfig
ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- make
```

```bash
U-Boot> version
U-Boot 2018.11-g208ecba (Nov 14 2018 - 13:17:50 -0800)

aarch64-linux-gnu-gcc (Linaro GCC 7.3-2018.05) 7.3.1 20180425 [linaro-7.3-2018.05 revision d29120a424ec
fbc167ef90065c0eeb7f91977701]
GNU ld (Linaro_Binutils-2018.05) 2.28.2.20170706
```

Environment:
TODO

## SD card

TODO

## Links

- [BCM2711](https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2711/README.md)
- [RPi4 datasheet](https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2711/rpi_DATA_2711_1p0_preliminary.pdf)
- [Revised BCM2837 doc](https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf)
- [Bare metal boot code for ARMv8-A](http://infocenter.arm.com/help/topic/com.arm.doc.dai0527a/DAI0527A_baremetal_boot_code_for_ARMv8_A_processors.pdf)
- [Bare-metal C++ code](https://github.com/rsta2/circle)
- [bcm2711-rpi-4-b.dtb](https://github.com/Hexxeh/rpi-firmware/blob/master/bcm2711-rpi-4-b.dtb)
- [Linux GENET drivers](https://github.com/torvalds/linux/tree/master/drivers/net/ethernet/broadcom/genet)
- [More info on RPi4](https://www.raspberrypi.org/forums/viewtopic.php?t=244479&start=25)
