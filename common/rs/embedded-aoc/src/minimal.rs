use core::ops;

use embedded_io::Read;

use crate::{
    BUFFER_SIZE, Day, Duration, END_INPUT_TAG, Handler, Instant, PartResult, START_INPUT_TAG,
    Timer, check_eof,
};

/// # Panics
#[allow(clippy::used_underscore_binding, clippy::uninlined_format_args)]
pub fn run<const NOM: u32, const DENOM: u32>(
    mut rx: impl Read,
    timer: &impl Timer<u64, NOM, DENOM>,
    mut handler: impl Handler<u64, NOM, DENOM>,
) -> !
where
    Instant<u64, NOM, DENOM>: ops::Sub<Output = Duration<u64, NOM, DENOM>>,
{
    let mut buffer = [0; BUFFER_SIZE];
    loop {
        let mut length = 0;
        loop {
            if length >= buffer.len() {
                break;
            }

            match rx.read(&mut buffer[length..]) {
                Err(_) => {
                    break;
                }
                Ok(0) => {
                    // ???
                }
                Ok(count) => {
                    let eof = check_eof(&buffer[length..length + count])
                        .map(|position| position + length);
                    length += count;

                    if let Some(eof) = eof {
                        if let Ok(input) = core::str::from_utf8(&buffer[..eof]) {
                            if let Some(start_position) = input.find(START_INPUT_TAG)
                                && let Some(end_position) = input.find(END_INPUT_TAG)
                            {
                                let Ok(day) =
                                    input[start_position + START_INPUT_TAG.len()..].parse::<Day>()
                                else {
                                    handler.unsupported_day();
                                    break;
                                };

                                let input = input
                                    [start_position + START_INPUT_TAG.len() + 2..end_position]
                                    .trim();

                                let mut part_1 = PartResult::new();
                                let mut part_2 = PartResult::new();

                                let start = timer.now();

                                handler.started(day, start);

                                if day.part_1(&mut part_1, input).is_err() {
                                    break;
                                }

                                if day.part_2(&mut part_2, input).is_err() {
                                    break;
                                }

                                let elapsed = timer.now() - start;
                                handler.ended(day, elapsed, part_1.as_str(), part_2.as_str());
                            } else {
                                handler.invalid_input();
                            }
                        } else {
                            handler.invalid_input();
                        }

                        break;
                    }
                }
            }
        }
    }
}
