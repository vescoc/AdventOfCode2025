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
[Install probe-rs][https://probe.rs/]

Example:
```bash
cargo binstall probe-rs-tools
```

### Install Rust nightly
targets:
- thumbv6m-none-eabi (rp-pico) 
- thumbv7em-none-eabihf (stm32 f3 discovery, stm32 h743zi nucleo, ...)
- thumbv6m-none-eabi (rp-pico2)

### rp-pico
#### Build for rp-pico
```bash
cargo +nightly build -r --target thumbv6m-none-eabi -p rp-pico -Z build-std=core
```

#### Run on rp-pico
```bash
cargo +nightly run -r --target thumbv6m-none-eabi -p rp-pico2 -Z build-std=core -- --chip RP2040
```

### rp-pico2
#### Build for rp-pico2
```bash
cargo +nightly build -r --target thumbv8m.main-none-eabihf -p rp-pico2 -Z build-std=core
```

#### Run on rp-pico2
```bash
cargo +nightly run -r --target thumbv8m.main-none-eabihf -p rp-pico2 -Z build-std=core -- --chip RP235x
```

### stm32f3-discovery
#### Build for stm32f3-discovery
```bash
cargo +nightly build -r --target thumbv7em-none-eabihf -p stm32f3-discovery -Z build-std=core
```

#### Run on stm32f3-discovery
```bash
cargo +nightly run -r --target thumbv7em-none-eabihf -p stm32f3-discovery -Z build-std=core -- --chip STM32F303VC
```

### stm32f411e-disco
#### Build for stm32f411e-disco
```bash
cargo +nightly build -r --target thumbv7em-none-eabihf -p stm32f411e-disco -Z build-std=core
```

#### Run on stm32f411e-disco
```bash
cargo +nightly run -r --target thumbv7em-none-eabihf -p stm32f411e-disco -Z build-std=core -- --chip STM32F411VE
```

### stm32h743zi-nucleo
#### Build for stm32h743zi-nucleo
```bash
cargo +nightly build -r --target thumbv7em-none-eabihf -p stm32h743zi-nucleo -Z build-std=core
```

#### Run on stm32h743zi-nucleo
```bash
cargo +nightly run -r --target thumbv7em-none-eabihf -p stm32h743zi-nucleo -Z build-std=core -- --chip STM32H743ZI
```

### nrf52840-dk
#### Build for nrf52840-dk
```bash
cargo +nightly build -r --target thumbv7em-none-eabihf -p nrf52840-dk -Z build-std=core
```

#### Run on nrf52840-dk
```bash
cargo +nightly run -r --target thumbv7em-none-eabihf -p nrf52840-dk -Z build-std=core -- --chip nRF52840_xxAA
```

### esp32
#### Build for esp32
```bash
cargo +esp build -r --target xtensa-esp32-none-elf -p aoc-esp32 -F esp32 -Z build-std=core --lib --bins
```

#### Run on esp32
Serial adapter:
- TX: esp32 PIN 18 (esp32 RX out)
- RX: esp32 PIN 19 (esp32 TX out)
- GND: esp32 GND

```bash
cargo +esp run -r --target xtensa-esp32-none-elf -p aoc-esp32 -F esp32 -Z build-std=core
```

### esp32s2 (mini)
#### Build for esp32s2 (mini)
```bash
cargo +esp build -r --target xtensa-esp32s2-none-elf -p aoc-esp32 -F esp32s2 -Z build-std=core --lib --bins
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

For flashing esp32s2 (mini): hold Boot button, bold Reset button, release Reset button, release Boot button, this make the MCU on boot mode.
```bash
cargo +esp run -r --target xtensa-esp32s2-none-elf -p aoc-esp32 -F esp32s2 -Z build-std=core -- --no-stub -b no-reset
```
After flashing, push Reset button for resetting.

### esp32s3
#### Build for esp32s3
```bash
cargo +esp build -r --target xtensa-esp32s3-none-elf -p aoc-esp32 -F esp32s3 -Z build-std=core --lib --bins
```

#### Run on esp32s3
Serial adapter:
- TX: esp32s3 PIN 18 (esp32s3 RX out)
- RX: esp32s3 PIN 17 (esp32s3 TX out)
- GND: esp32s3 GND

```bash
cargo +esp run -r --target xtensa-esp32s3-none-elf -p aoc-esp32 -F esp32s3 -Z build-std=core
```

### esp32c3
#### Build for esp32c3
```bash
cargo +nightly build -r --target riscv32imc-unknown-none-elf -p aoc-esp32 -F esp32c3 -Z build-std=core --lib --bins
```

#### Run on esp32c3
Serial adapter:
- TX: esp32c3 PIN 18 (esp32c3 RX out)
- RX: esp32c3 PIN 19 (esp32c3 TX out)
- GND: esp32 GND

```bash
cargo +nightly run -r --target riscv32imc-unknown-none-elf -p aoc-esp32 -F esp32c3 -Z build-std=core
```

### esp32c6
#### Build for esp32c6
```bash
cargo +nightly build -r --target riscv32imac-unknown-none-elf -p aoc-esp32 -F esp32c6 -Z build-std=core --lib --bins
```

#### Run on esp32c6
Serial adapter:
- TX: esp32c6 PIN 18 (esp32c6 RX out)
- RX: esp32c6 PIN 19 (esp32c6 TX out)
- GND: esp32 GND

```bash
cargo +nightly run -r --target riscv32imac-unknown-none-elf -p aoc-esp32 -F esp32c6 -Z build-std=core
```
