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
  cookie (esample: `53616...`).

## Building wasm32-unknown-unknown
```bash
cd days/dayXY/rsui
trunk build --release --filehash false --public-url /AdventOfCode2025/dayXY
```

