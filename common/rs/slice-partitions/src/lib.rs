#![no_std]

use core::{mem, ops};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid partitions specification")]
    Partitions,

    #[error("Insufficient data")]
    Len,
}

/// # Errors
pub fn partition<'a, const SIZE: usize, T, U>(
    data: &'a mut [T],
    partitions: &[ops::Range<usize>],
    f: impl FnOnce(&mut [&'a mut [T]]) -> U,
) -> Result<U, Error> {
    let len = partitions.len();
    if len > SIZE {
        return Err(Error::Len);
    }

    if len == 0 {
        return Ok(f(&mut []));
    }

    for (i, p1) in partitions.iter().enumerate().take(partitions.len() - 1) {
        for p2 in partitions.iter().skip(i + 1) {
            if p1.contains(&p2.start)
                || p1.contains(&p2.end)
                || p2.contains(&p1.start)
                || p2.contains(&p2.end)
                || (p1.start <= p2.start && p1.end >= p2.end)
                || (p2.start <= p1.start && p2.end >= p1.end)
            {
                return Err(Error::Partitions);
            }
        }
    }

    if partitions
        .iter()
        .any(|ops::Range { end, .. }| *end > data.len())
    {
        return Err(Error::Partitions);
    }

    unsafe { Ok(partition_unsafe::<SIZE, T, U>(data, partitions, f)) }
}

/// # Safety
/// - partitions must not overlapping
pub unsafe fn partition_unsafe<'a, const SIZE: usize, T, U>(
    data: &'a mut [T],
    partitions: &[ops::Range<usize>],
    f: impl FnOnce(&mut [&'a mut [T]]) -> U,
) -> U {
    let data = data.as_mut_ptr();
    let mut vector = [const { mem::MaybeUninit::<&'a mut [T]>::uninit() }; SIZE];
    for (ops::Range { start, end }, e) in partitions.iter().zip(vector.iter_mut()) {
        e.write(unsafe { core::slice::from_raw_parts_mut(data.add(*start), end - start) });
    }

    let vector = &mut vector[..partitions.len()];
    f(unsafe { &mut *(core::ptr::from_mut(vector) as *mut [&mut [T]]) })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_partitions() {
        #[rustfmt::skip]
        let mut data = [1.0f32, 1.0, 2.0,
                        1.0, -1.0, 0.0];

        partition::<2, _, _>(&mut data, &[0..3, 3..6], |eqs| {
            let Some((eq1, [eq2, ..])) = eqs.split_first_mut() else { unreachable!(); };
            for (b, a) in eq2.iter_mut().zip(eq1.iter()) {
                *b -= a;
                *b /= -2.0;
            }
            for (a, b) in eq1.iter_mut().zip(eq2.iter()) {
                *a -= b;
            }
        })
        .unwrap();

        assert!(
            data.iter()
                .zip(&[1.0, 0.0, 1.0, 0.0, 1.0, 1.0])
                .all(|(a, b)| (a - b).abs() < 1e-6),
            "{data:?}",
        );
    }    

    #[test]
    fn test_invalid_partitions() {
        #[rustfmt::skip]
        let mut data = [1.0f32, 1.0, 2.0,
                        1.0, -1.0, 0.0];

        let mut invalid = false;
        assert_eq!(
            partition::<2, _, _>(&mut data, &[0..4, 3..6], |_| {
                invalid = true;
            }),
            Err(Error::Partitions),
        );

        assert!(!invalid);
    }    

    #[test]
    fn test_insufficient_data() {
        #[rustfmt::skip]
        let mut data = [1.0f32, 1.0, 2.0,
                        1.0, -1.0, 0.0];

        let mut invalid = false;
        assert_eq!(
            partition::<2, _, _>(&mut data, &[0..3, 3..6, 6..9], |_| {
                invalid = true;
            }),
            Err(Error::Len),
        );

        assert!(!invalid);
    }    
}
