# AdventOfCode2025
https://adventofcode.com/2025

## Build
- get session cookie from https://adventofcode.com/
- create file `$HOME/.config/adventofcode.session` (Linux, example:
  `/home/alice/.config/adventofcode.session`) or
  `$HOME/Library/Application Support/adventofcode.session` (macOS,
  example: `/Users/Alice/Library/Application
  Support/adventofcode.session`) or `{FOLDERID_RoamingAppData}`
  (Windows, example:
  `C:\Users\Alice\AppData\Roamingadventofcode.session`) and copy the
  cookie (esample: `53616...`)
- Testing a single day:
```bash
cargo test -p dayXY
```
- Running a single day (release):
```bash
cargo run -p dayXY -r
```
- Bench a single day:
```bash
cargo bench -p dayXY
```

## Building wasm32-unknown-unknown
```bash
cd days/dayXY/rsui && trunk build --release --filehash false --public-url /AdventOfCode2025/dayXY
```

## Embedded
### Install probe-rs-tools
For flashing / running on rp-pico, rp-pico2, nRF52840-dk, stm32f3-discovery, stm32f4-disco
and stm32h741zi-nucleo [install probe-rs](https://probe.rs/).

Example:
```bash
cargo binstall probe-rs-tools
```

### Install espflash
For flashing / running on esp32, esp32s2, esp32s3, esp32c3, esp32c6
[install espflash](https://github.com/esp-rs/espflash/blob/main/espflash/README.md).

Example:
```bash
cargo binstall espflash
```

### Install Rust nightly
targets:
- thumbv6m-none-eabi (rp-pico) 
- thumbv7em-none-eabihf (stm32 f3 discovery, stm32 h743zi nucleo, nRF 52840, ...)
- thumbv6m-none-eabi (rp-pico2)
- riscv32imc-unknown-none-elf (esp32-c3)
- riscv32imac-unknown-none-elf (esp32-c6)

### Install Rust esp32 toolkit
[Install espup](https://github.com/esp-rs/espup)

targets:
- xtensa-esp32-none-elf (esp32)
- xtensa-esp32s2-none-elf (esp32-s2)
- xtensa-esp32s3-none-elf (esp32-s3)

### How to invoke the MCU
1. Prepare a file like this:
```raw
START INPUT DAY: XY
<data>
END INPUT
```
Example data from day 1, file `day01-example.txt`:
```raw
START INPUT DAY: 01
L68
L30
R48
L5
R60
L55
L1
L99
R14
L82
END INPUT
```
2. Identify the serial device. Example for linux: `/dev/ttyUSB0` or `/dev/ttyACM0`.
3. Configure the serial device (maybe check if you need root/admin permissions).
For some MCU you need a serial adapter.
Example (linux):
```bash
stty -F /dev/ttyUSB0 raw 115200
```
4. Get data from the serial device.
Example (linux):
```bash
cat /dev/ttyUSB0
```
5. Send the data to the serial device.
Example (linux):
```bash
cat data01-example.txt > /dev/ttyUSB0
```
6. See the results at point 4.
7. Go to point 5 with other formatted input data for the same or other AoC day.
8. If the device does not respond in about max ten seconds probably it crashed.
Check your input data. Reset the device and retry. If you have problem with your data,
please create an issue.
8. Enjoy.

### rp-pico
- target CPU: Cortex-M0+
- system clock: 200Mhz with feature overclock, default 124Mhz

#### Build for rp-pico
```bash
cargo +nightly --config target.thumbv6m-none-eabi.rustflags='["-C","target-cpu=cortex-m0plus"]' build-rp-pico
```

#### Run on rp-pico (125Mhz)
```bash
cargo +nightly --config target.thumbv6m-none-eabi.rustflags='["-C","target-cpu=cortex-m0plus"]' run-rp-pico
```
#### Run on rp-pico overclock (200Mhz)
```bash
cargo +nightly --config target.thumbv6m-none-eabi.rustflags='["-C","target-cpu=cortex-m0plus"]' run-rp-pico-overclock
```

### rp-pico2
- target CPU: Cortex-M33
- system clock: 290Mhz with feature overclock, default 150Mhz

#### Build for rp-pico2
```bash
cargo +nightly --config target.thumbv8m.main-none-eabihf.rustflags='["-C","target-cpu=cortex-m33"]' build-rp-pico2
```

#### Run on rp-pico2 (150Mhz)
```bash
cargo +nightly --config target.thumbv8m.main-none-eabihf.rustflags='["-C","target-cpu=cortex-m33"]' run-rp-pico2
```

#### Run on rp-pico2 overclock (290Mhz)
```bash
cargo +nightly --config target.thumbv8m.main-none-eabihf.rustflags='["-C","target-cpu=cortex-m33"]' run-rp-pico2-overclock
```

### stm32f3-discovery
- target CPU: Cortex-M4F
- system clock: 72Mhz

#### Build for stm32f3-discovery
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m4"]' build-stm32f3-discovery
```

#### Run on stm32f3-discovery
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m4"]' run-stm32f3-discovery
```

### stm32f411e-disco
- target CPU: Cortex-M4F
- system clock: 96Mhz

#### Build for stm32f411e-disco
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m4"]' build-stm32f411e-disco
```

#### Run on stm32f411e-disco
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m4"]' run-stm32f411e-disco
```

### stm32h743zi-nucleo
- target CPU: Cortex-M7F
- system clock: 400Mhz

#### Build for stm32h743zi-nucleo
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m7"]' build-stm32h743zi-nucleo
```

#### Run on stm32h743zi-nucleo
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m7"]' run-stm32h743zi-nucleo
```

### nrf52840-dk
- target CPU: Cortex-M4F
- system clock: 64Mhz

#### Build for nrf52840-dk
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m4"]' build-nrf52840-dk
```

#### Run on nrf52840-dk
```bash
cargo +nightly --config target.thumbv7em-none-eabihf.rustflags='["-C","target-cpu=cortex-m4"]' run-nrf52840-dk
```

### esp32
#### Build for esp32
```bash
cargo +esp --config target.xtensa-esp32-none-elf.rustflags='["-C","target-cpu=esp32"]' build-esp32
```

#### Run on esp32
Serial adapter:
- TX: esp32 PIN 18 (esp32 RX out)
- RX: esp32 PIN 19 (esp32 TX out)
- GND: esp32 GND

```bash
cargo +esp --config target.xtensa-esp32-none-elf.rustflags='["-C","target-cpu=esp32"]' run-esp32
```

### esp32s2 (mini)
#### Build for esp32s2 (mini)
```bash
cargo +esp --config target.xtensa-esp32s2-none-elf.rustflags='["-C","target-cpu=esp32s2"]' build-esp32s2
```

#### Run on esp32s2 (mini)
Serial adapter:
- TX: esp32s2 PIN 18 (esp32s2 RX out)
- RX: esp32s2 PIN 17 (esp32s2 TX out)
- GND: esp32s2 GND

Logging serial adapter:
- TX: esp32s2 PIN 39 (esp32s2 RX out)
- RX: esp32s2 PIN 40 (esp32s2 TX out)
- GND: esp32s2 GND
- Led at PIN 15 on when there is an error

For flashing esp32s2 (mini): hold Boot button, hold Reset button, release Reset button, release Boot button, this make the MCU on boot mode.
```bash
cargo +esp --config target.xtensa-esp32s2-none-elf.rustflags='["-C","target-cpu=esp32s2"]' run-esp32s2 -- --no-stub -b no-reset
```
After flashing, push Reset button for resetting.

For logging identify the second serial adapter (Logging serial adapter), example `/dev/ttyUSB1`, and set (maybe you need root/admin permissions):
```bash
stty -F /dev/ttyUSB1 raw 115200
cat /dev/ttyUSB1
```

### esp32s3
#### Build for esp32s3
```bash
cargo +esp --config target.xtensa-esp32s3-none-elf.rustflags='["-C","target-cpu=esp32s3"]' build-esp32s3
```

#### Run on esp32s3
Serial adapter:
- TX: esp32s3 PIN 18 (esp32s3 RX out)
- RX: esp32s3 PIN 17 (esp32s3 TX out)
- GND: esp32s3 GND

```bash
cargo +esp --config target.xtensa-esp32s3-none-elf.rustflags='["-C","target-cpu=esp32s3"]' run-esp32s3
```

### esp32c3
#### Build for esp32c3
```bash
cargo +nightly --config target.riscv32imc-unknown-none-elf.rustflags='["-C","target-cpu=generic-rv32"]' build-esp32c3
```

#### Run on esp32c3
Serial adapter:
- TX: esp32c3 PIN 18 (esp32c3 RX out)
- RX: esp32c3 PIN 19 (esp32c3 TX out)
- GND: esp32 GND

```bash
cargo +nightly --config target.riscv32imc-unknown-none-elf.rustflags='["-C","target-cpu=generic-rv32"]' run-esp32c3
```

### esp32c6
#### Build for esp32c6
```bash
cargo +nightly --config target.riscv32imac-unknown-none-elf.rustflags='["-C","target-cpu=generic-rv32"]' build-esp32c6
```

#### Run on esp32c6
Serial adapter:
- TX: esp32c6 PIN 18 (esp32c6 RX out)
- RX: esp32c6 PIN 19 (esp32c6 TX out)
- GND: esp32 GND

```bash
cargo +nightly --config target.riscv32imac-unknown-none-elf.rustflags='["-C","target-cpu=generic-rv32"]' run-esp32c6
```
