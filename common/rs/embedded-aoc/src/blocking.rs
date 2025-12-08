use core::ops;

use embedded_io::{Read, Write};

use crate::{
    Day, Duration, END_INPUT_TAG, Handler, Instant, PartResult, START_INPUT_TAG, Timer, info,
    trace, warn,
};

/// # Panics
#[allow(clippy::used_underscore_binding, clippy::uninlined_format_args)]
pub fn run<const NOM: u32, const DENOM: u32>(
    (mut rx, mut tx): (impl Read, impl Write),
    timer: &impl Timer<u64, NOM, DENOM>,
    mut handler: impl Handler<u64, NOM, DENOM>,
) -> !
where
    Instant<u64, NOM, DENOM>: ops::Sub<Output = Duration<u64, NOM, DENOM>>,
{
    trace!("run");

    let mut buffer = [0; 25 * 1024];
    loop {
        let mut length = 0;
        loop {
            if length >= buffer.len() {
                warn!("buffer overflow");
                break;
            }

            match rx.read(&mut buffer[length..]) {
                Err(_err) => {
                    #[cfg(feature = "log")]
                    warn!("error reading: {_err:?}");
                }
                Ok(0) => {
                    trace!("reading 0 bytes");
                }
                Ok(count) => {
                    debug_assert!(length + count <= buffer.len(), "invalid count");

                    length += count;

                    if let Ok(input) = core::str::from_utf8(&buffer[..length]) {
                        match (input.find(START_INPUT_TAG), input.find(END_INPUT_TAG)) {
                            (Some(start_position), Some(end_position)) => {
                                let Ok(day) =
                                    input[start_position + START_INPUT_TAG.len()..].parse::<Day>()
                                else {
                                    warn!("unsupported day");

                                    handler.unsupported_day();

                                    write!(&mut tx, "unsupported day\r\n").ok();

                                    break;
                                };

                                let input = input
                                    [start_position + START_INPUT_TAG.len() + 2..end_position]
                                    .trim();

                                info!("[{}] start working on {}", day, day);

                                let mut part_1 = PartResult::new();
                                let mut part_2 = PartResult::new();

                                let start = timer.now();

                                handler.started(day, start);

                                if day.part_1(&mut part_1, input).is_err() {
                                    warn!("part_1: buffer overflow");
                                    break;
                                }

                                if day.part_2(&mut part_2, input).is_err() {
                                    warn!("part_2: buffer overflow");
                                    break;
                                }

                                let elapsed = timer.now() - start;

                                handler.ended(day, elapsed, part_1.as_str(), part_2.as_str());

                                info!("[{}] part 1: {}", day, part_1.as_str());
                                write!(&mut tx, "[{day}] part 1: {part_1}\r\n").ok();

                                info!("[{}] part 2: {}", day, part_2.as_str());
                                write!(&mut tx, "[{day}] part 2: {part_2}\r\n").ok();

                                info!(
                                    "[{}] elapsed: {}ms ({}µs)",
                                    day,
                                    elapsed.to_millis(),
                                    elapsed.to_micros()
                                );
                                write!(
                                    &mut tx,
                                    "[{day}] elapsed: {}ms ({}µs)\r\n",
                                    elapsed.to_millis(),
                                    elapsed.to_micros()
                                )
                                .ok();

                                break;
                            }
                            (None, Some(_)) => {
                                warn!("invalid input");

                                handler.invalid_input();

                                write!(&mut tx, "invalid input\r\n").ok();

                                break;
                            }
                            _ => {}
                        }
                    } else {
                        warn!("invalid utf8 data");
                        break;
                    }
                }
            }
        }
    }
}
