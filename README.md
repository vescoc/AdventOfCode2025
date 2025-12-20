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

## rp-pico
### Build for rp-pico
```bash
cargo +nightly build -r --target thumbv6m-none-eabi -p rp-pico -Z build-std=core
```

### Run on rp-pico
```bash
cargo +nightly run -r --target thumbv6m-none-eabi -p rp-pico2 -Z build-std=core
```

## rp-pico2
### Build for rp-pico2
```bash
cargo +nightly build -r --target thumbv8m.main-none-eabihf -p rp-pico2 -Z build-std=core
```

### Run on rp-pico2
```bash
cargo +nightly run -r --target thumbv8m.main-none-eabihf -p rp-pico2 -Z build-std=core
```
