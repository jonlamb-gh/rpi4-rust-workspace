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
U-Boot 2019.10-rc4-00004-g1f3910d (Sep 26 2019 - 18:01:19 -0700)

aarch64-linux-gnu-gcc (Linaro GCC 7.3-2018.05) 7.3.1 20180425 [linaro-7.3-2018.05 revision d29120a424ec
fbc167ef90065c0eeb7f91977701]
GNU ld (Linaro_Binutils-2018.05) 2.28.2.20170706
```

Environment:

```bash
setenv imgname img.bin

# Normally for bare metal
#setenv loadaddr 0x80000

# Put it somewhere else, so we don't overwrite u-boot
setenv loadaddr 0x01000000

# Disable data cache because u-boot turns it on
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
- [More info on RPi4](https://www.raspberrypi.org/forums/viewtopic.php?t=244479&start=25)
- [U-boot load over serial with kermit](http://blog.mezeske.com/?p=483)
- [My RPi3 workspace](https://github.com/jonlamb-gh/rpi3-rust-workspace)
