use core::mem;
use core::ops;

use crate::LOG;

const EPS: f64 = 1e-6;

pub trait BaseMap {
    fn base(&self, x: usize) -> Option<usize>;
    fn base_in(&mut self, x: usize, n: usize);
    fn base_out(&mut self, x: usize);
    fn base_for_equation(&self, n: usize) -> Option<usize>;
    fn is_base(&self, x: usize) -> bool;
    fn reset(&mut self);
}

impl<const SIZE: usize> BaseMap for [Option<usize>; SIZE] {
    fn base(&self, x: usize) -> Option<usize> {
        self.get(x).copied().flatten()
    }

    fn base_in(&mut self, x: usize, n: usize) {
        self[x].replace(n);
    }

    fn base_out(&mut self, x: usize) {
        self[x] = None;
    }

    fn base_for_equation(&self, n: usize) -> Option<usize> {
        self.iter().position(|e| matches!(e, Some(v) if *v == n))
    }

    fn is_base(&self, x: usize) -> bool {
        self.get(x).is_some_and(Option::is_some)
    }

    fn reset(&mut self) {
        for e in self.iter_mut() {
            *e = None;
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Partitions,
    Len,
}

/// # Panics
pub fn simplex_eqs<T>(
    bases: &mut (impl BaseMap + core::fmt::Debug),
    eqs: &mut [&mut [f64]],
    tags: Option<&mut [T]>,
) -> Option<f64> {
    if LOG {
        println!("start");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    bases.reset();

    if !reduce(bases, &mut eqs[1..], tags) {
        return None;
    }

    if LOG {
        println!("reduce {bases:?}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    if !change_bases(bases, &mut eqs[1..]) {
        return None;
    }

    if LOG {
        println!("change bases {bases:?}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    {
        let (eqc, eqs) = eqs.split_first_mut().unwrap();
        tableau(bases, eqc, eqs);
    }

    if LOG {
        println!("tableau {bases:?}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    let len = eqs[0].len();
    while let Some(base_in) = eqs[0].iter().take(len - 1).enumerate().find_map(|(j, v)| {
        if *v < -EPS && !bases.is_base(j) {
            Some(j)
        } else {
            None
        }
    }) {
        if LOG {
            println!("working on {base_in}");
        }

        let (i, _) = find(&eqs[1..], base_in).expect("???");
        let base_out = bases.base_for_equation(i).expect("cannot find base on {i}");

        assert!(base_in != base_out);

        pivot(eqs, i + 1, base_in);

        bases.base_out(base_out);
        bases.base_in(base_in, i);

        if LOG {
            println!("pivot {bases:?} from {base_out} to {base_in}");
            for eq in eqs.iter() {
                println!("{eq:?}");
            }
        }
    }

    if LOG {
        println!("done {bases:?}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    Some(-eqs[0][len - 1])
}

/// # Errors
pub fn simplex<const SIZE: usize, T>(
    bases: &mut (impl BaseMap + core::fmt::Debug),
    data: &mut [f64],
    partitions: &[ops::Range<usize>],
    tags: Option<&mut [T]>,
) -> Result<Option<f64>, Error> {
    partition::<SIZE, _, _>(data, partitions, move |eqs| simplex_eqs(bases, eqs, tags))
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
    f(unsafe {
        // mem::transmute::<&mut [core::mem::MaybeUninit<&mut [T]>], &mut [&mut [T]]>(vector)
        &mut *(core::ptr::from_mut(vector) as *mut [&mut [T]])
    })
}

fn change_bases(bases: &mut impl BaseMap, equations: &mut [&mut [f64]]) -> bool {
    while let Some(((i, new_base), _)) = equations
        .iter()
        .enumerate()
        .filter_map(|(n, eq)| {
            bases.base_for_equation(n).and_then(|_| {
                eq.last().and_then(|&b| {
                    if b < -EPS {
                        eq.iter()
                            .take(eq.len() - 1)
                            .enumerate()
                            .filter_map(|(i, &v)| {
                                if !bases.is_base(i) && v < -EPS {
                                    Some(((n, i), b / v))
                                } else {
                                    None
                                }
                            })
                            .max_by(|&(_, a), &(_, b)| (a).partial_cmp(&b).unwrap())
                    } else {
                        None
                    }
                })
            })
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    {
        let old_base = bases.base_for_equation(i).expect("cannot find base on {i}");

        assert!(old_base != new_base);

        pivot(equations, i, new_base);

        bases.base_out(old_base);
        bases.base_in(new_base, i);

        if LOG {
            println!("change base from {old_base} to {new_base}");
        }
    }

    equations
        .iter()
        .enumerate()
        .all(|(n, eq)| bases.base_for_equation(n).is_none() || eq[eq.len() - 1] > -EPS)
}

fn pivot(equations: &mut [&mut [f64]], i: usize, j: usize) {
    equations.swap(i, 0);

    let (first, eqs) = equations.split_first_mut().unwrap();

    let d = first[j];
    for e in first.iter_mut() {
        *e /= d;
    }

    for eq in eqs.iter_mut() {
        let mul = eq[j];
        for (e, v) in eq.iter_mut().zip(first.iter()) {
            *e -= mul * v;
        }
    }

    equations.swap(i, 0);
}

fn find(equations: &[&mut [f64]], k: usize) -> Option<(usize, f64)> {
    let l = equations[0].len();
    equations
        .iter()
        .enumerate()
        .filter_map(|(i, eq)| {
            let aik = eq[k];
            if aik > EPS {
                Some((i, eq[l - 1] / aik))
            } else {
                None
            }
        })
        .min_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap())
}

fn tableau(bases: &impl BaseMap, eqc: &mut [f64], equations: &[&mut [f64]]) {
    for j in 0..eqc.len() - 1 {
        if let Some(eq) = bases.base(j).and_then(|k| equations.get(k)) {
            let mul = eqc[j];
            for (e, v) in eqc.iter_mut().zip(eq.iter()) {
                *e -= mul * v;
            }
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn reduce<T>(
    bases: &mut impl BaseMap,
    equations: &mut [&mut [f64]],
    mut tags: Option<&mut [T]>,
) -> bool {
    let n = equations.len();
    let k = equations[0].len();

    let mut i = 0;
    let mut j = 0;
    while i < n && j < k - 1 {
        let Some(r) = equations
            .iter()
            .enumerate()
            .position(|(candidate, equation)| candidate >= i && equation[j] != 0.0)
        else {
            j += 1;
            continue;
        };

        bases.base_in(j, i);

        equations.swap(r, 0);
        if let Some(tags) = tags.as_mut() {
            tags.swap(r, 0);
        }

        let value = equations[0][j];
        for e in equations[0].iter_mut() {
            *e /= value;
        }

        let (first, others) = equations.split_first_mut().unwrap();
        for eq in others.iter_mut() {
            let mul = eq[j] / first[j];
            for (e, v) in eq.iter_mut().zip(first.iter()) {
                *e -= v * mul;
            }
        }

        equations.swap(r, 0);
        if let Some(tags) = tags.as_mut() {
            tags.swap(r, 0);
        }

        equations.swap(r, i);
        if let Some(tags) = tags.as_mut() {
            tags.swap(r, i);
        }

        i += 1;
        j += 1;
    }

    equations
        .iter()
        .all(|eq| eq[k - 1].abs() < EPS || eq.iter().take(k - 1).any(|v| v.abs() >= EPS))
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_eq_float {
        ($e1:expr, $e2:expr) => {
            assert!(($e1 - $e2).abs() < EPS, "{} != {}", $e1, $e2);
        };
    }

    #[test]
    fn test_reduce_1() {
        let mut eq1 = [1.0, 1.0, 2.0];
        let mut eq2 = [1.0, -1.0, 0.0];
        let mut eqs = [eq1.as_mut_slice(), eq2.as_mut_slice()];

        let mut tags = [0, 1];
        let mut bases = [None; 16];
        reduce(&mut bases, &mut eqs, Some(&mut tags));

        assert_eq!(
            eqs,
            [[1.0, 0.0, 1.0].as_slice(), [0.0, 1.0, 1.0].as_slice()]
        );
        assert_eq!(&tags, &[0, 1]);
        assert_eq!(&bases[..2], &[Some(0), Some(1)]);
    }

    #[test]
    fn test_reduce_2() {
        let mut eq1 = [0.0, 2.0, 2.0];
        let mut eq2 = [2.0, 0.0, 2.0];
        let mut eqs = [eq1.as_mut_slice(), eq2.as_mut_slice()];

        let mut tags = [0, 1];
        let mut bases = [None; 16];
        reduce(&mut bases, &mut eqs, Some(&mut tags));

        assert_eq!(
            eqs,
            [[1.0, 0.0, 1.0].as_slice(), [0.0, 1.0, 1.0].as_slice()]
        );
        assert_eq!(&tags, &[1, 0]);
        assert_eq!(&bases[..2], &[Some(0), Some(1)]);
    }

    #[test]
    fn test_simplex_4() {
        let mut bases = [None; 4];

        #[rustfmt::skip]
        let mut data = [
            1.0, 1.0, 1.0, 1.0, 0.0,
            0.0, 1.0, 0.0, 0.0, 5.0,
            1.0, 0.0, 1.0, 1.0, 38.0,
            1.0, 0.0, 0.0, 0.0, 18.0,
            0.0, 0.0, 1.0, 1.0, 20.0,
            0.0, 1.0, 0.0, 0.0, 5.0,
            0.0, 0.0, 0.0, 1.0, 14.0,
        ];

        let mut tags = [0, 1, 2, 3, 4, 5, 6];
        assert_eq!(
            simplex::<10, _>(
                &mut bases,
                &mut data,
                &[0..5, 5..10, 10..15, 15..20, 20..25, 25..30, 30..35,],
                Some(&mut tags),
            ),
            Ok(Some(43.0)),
        );
    }

    #[test]
    fn test_simplex() {
        let mut bases = [None; 6];

        #[rustfmt::skip]
        let mut data = [
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 3.0,
            0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 5.0,
            0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 4.0,
            1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0,
        ];
        let mut tags = [0, 1, 2, 3, 4];

        assert_eq!(
            simplex::<10, _>(
                &mut bases,
                &mut data,
                &[0..7, 7..14, 14..21, 21..28, 28..35,],
                Some(&mut tags),
            ),
            Ok(Some(10.0)),
        );
    }

    #[test]
    fn test_simplex_6() {
        let mut bases = [None; 5];

        #[rustfmt::skip]
        let mut data = [
            1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
            1.0, 0.0, 1.0, 1.0, 0.0, 7.0,
            0.0, 0.0, 0.0, 1.0, 1.0, 5.0,
            1.0, 1.0, 0.0, 1.0, 1.0, 12.0,
            1.0, 1.0, 0.0, 0.0, 1.0, 7.0,
            1.0, 0.0, 1.0, 0.0, 1.0, 2.0,
        ];
        let mut tags = [0, 1, 2, 3, 4, 5];

        assert_eq!(
            simplex::<10, _>(
                &mut bases,
                &mut data,
                &[0..6, 6..12, 12..18, 18..24, 24..30, 30..36,],
                Some(&mut tags),
            ),
            Ok(Some(12.0)),
        );
    }

    #[test]
    fn test_find_1() {
        let mut eq0 = [1.5, 1.0, 1.0, 0.0, 0.0, 27.0];
        let mut eq1 = [1.0, 1.0, 0.0, 1.0, 0.0, 21.0];
        let mut eq2 = [0.3, 0.5, 0.0, 0.0, 1.0, 9.0];
        let eqs = [eq0.as_mut_slice(), eq1.as_mut_slice(), eq2.as_mut_slice()];

        assert_eq!(find(&eqs, 0), Some((0, 18.0)));
    }

    #[test]
    fn test_simplex_ex() {
        let mut bases = [None; 5];

        #[rustfmt::skip]
        let mut data = [
            -130.0, -100.0, 0.0, 0.0, 0.0, 0.0,
            1.5, 1.0, 1.0, 0.0, 0.0, 27.0,
            1.0, 1.0, 0.0, 1.0, 0.0, 21.0,
            0.3, 0.5, 0.0, 0.0, 1.0, 9.0,
        ];

        let mut tags = [0, 1, 2, 3];

        assert_eq!(
            simplex::<10, _>(
                &mut bases,
                &mut data,
                &[0..6, 6..12, 12..18, 18..24,],
                Some(&mut tags),
            ),
            Ok(Some(-2460.0)),
        );

        assert!((data[6 * (bases.base(0).unwrap() + 1) + 5] - 12.0).abs() < EPS);
        assert!((data[6 * (bases.base(1).unwrap() + 1) + 5] - 9.0).abs() < EPS);
    }

    #[test]
    fn test_pivot_2() {
        let mut eqc = [-130.0, -100.0, 0.0, 0.0, 0.0, 0.0];
        let mut eq0 = [1.5, 1.0, 1.0, 0.0, 0.0, 27.0];
        let mut eq1 = [1.0, 1.0, 0.0, 1.0, 0.0, 21.0];
        let mut eq2 = [0.3, 0.5, 0.0, 0.0, 1.0, 9.0];
        let mut eqs = [
            eqc.as_mut_slice(),
            eq0.as_mut_slice(),
            eq1.as_mut_slice(),
            eq2.as_mut_slice(),
        ];

        pivot(&mut eqs, 1, 0);

        let target = [
            &[0.0, -40.0 / 3.0, 260.0 / 3.0, 0.0, 0.0, 2340.0],
            &[1.0, 2.0 / 3.0, 2.0 / 3.0, 0.0, 0.0, 18.0],
            &[0.0, 1.0 / 3.0, -2.0 / 3.0, 1.0, 0.0, 3.0],
            &[0.0, 0.3, -0.2, 0.0, 1.0, 3.6],
        ];

        for (a, b) in eqs.iter().zip(target.iter()) {
            for (a, b) in a.iter().zip(b.iter()) {
                assert!((a - b).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_find_2() {
        let mut eq0 = [1.5, 1.0, 1.0, 0.0, 0.0, 27.0];
        let mut eq1 = [1.0, 1.0, 0.0, 1.0, 0.0, 21.0];
        let mut eq2 = [0.3, 0.5, 0.0, 0.0, 1.0, 9.0];
        let eqs = [eq0.as_mut_slice(), eq1.as_mut_slice(), eq2.as_mut_slice()];

        assert_eq!(find(&eqs, 1), Some((2, 18.0)));
    }

    // [.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}
    #[test]
    fn test_simplex_7() {
        let mut bases = [None; 4];

        #[rustfmt::skip]
        let mut data = [
            1.0, 1.0, 1.0, 1.0, 0.0,
            1.0, 0.0, 1.0, 0.0, 10.0,
            1.0, 0.0, 1.0, 1.0, 11.0,
            1.0, 0.0, 1.0, 1.0, 11.0,
            1.0, 1.0, 0.0, 0.0, 5.0,
            1.0, 1.0, 1.0, 0.0, 10.0,
            0.0, 0.0, 1.0, 0.0, 5.0,
        ];

        let mut tags = [0, 1, 2, 3, 4, 5, 6];

        assert_eq!(
            simplex::<10, _>(
                &mut bases,
                &mut data,
                &[0..5, 5..10, 10..15, 15..20, 20..25, 25..30, 30..35,],
                Some(&mut tags),
            ),
            Ok(Some(11.0)),
        );
    }

    #[test]
    fn test_partitions() {
        #[rustfmt::skip]
        let mut data = [1.0, 1.0, 2.0,
                        1.0, -1.0, 0.0];

        let mut bases = [None; 2];
        let mut tags = [0, 1];
        partition::<2, _, _>(&mut data, &[0..3, 3..6], |eqs| {
            reduce(&mut bases, eqs, Some(&mut tags));
        })
        .unwrap();

        assert!(
            data.iter()
                .zip(&[1.0, 0.0, 1.0, 0.0, 1.0, 1.0])
                .all(|(a, b)| (a - b).abs() < EPS)
        );
    }

    #[test]
    fn test_simplex_i_p0() {
        let mut bases = [None; 4];

        #[rustfmt::skip]
        let mut data = [
            -5.0, -17.0 / 4.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 45.0,
        ];

        assert_eq!(
            simplex::<10, ()>(&mut bases, &mut data, &[0..5, 5..10, 10..15], None,),
            Ok(Some(-24.0625)),
        );

        assert_eq_float!(data[5 * (bases.base(0).unwrap() + 1) + 4], 3.75);
        assert_eq_float!(data[5 * (bases.base(1).unwrap() + 1) + 4], 1.25);
    }

    #[test]
    fn test_simplex_i_p1() {
        let mut bases = [None; 5];

        #[rustfmt::skip]
        let mut data = [
            -5.0, -17.0 / 4.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 0.0, 45.0,
            1.0, 0.0, 0.0, 0.0, 1.0, 3.0,
        ];

        assert_eq!(
            simplex::<10, ()>(&mut bases, &mut data, &[0..6, 6..12, 12..18, 18..24], None,),
            Ok(Some(-23.5)),
        );

        assert_eq_float!(data[6 * (bases.base(0).unwrap() + 1) + 5], 3.0);
        assert_eq_float!(data[6 * (bases.base(1).unwrap() + 1) + 5], 2.0);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_simplex_i_p2() {
        let mut bases = [None; 5];

        #[rustfmt::skip]
        let mut data = [
            -5.0, -17.0 / 4.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 0.0, 45.0,
            1.0, 0.0, 0.0, 0.0, -1.0, 4.0,
        ];

        assert_eq!(
            simplex::<10, ()>(&mut bases, &mut data, &[0..6, 6..12, 12..18, 18..24], None,),
            Ok(Some(-23.541666666666668)),
        );

        assert_eq_float!(data[6 * (bases.base(0).unwrap() + 1) + 5], 4.0);
        assert_eq_float!(
            data[6 * (bases.base(1).unwrap() + 1) + 5],
            0.8333333333333334
        );
    }

    #[test]
    fn test_simplex_i_p3() {
        let mut bases = [None; 6];

        #[rustfmt::skip]
        let mut data = [
            -5.0, -17.0 / 4.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 0.0, 0.0, 45.0,
            1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 4.0,
            0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ];

        assert_eq!(
            simplex::<10, ()>(
                &mut bases,
                &mut data,
                &[0..7, 7..14, 14..21, 21..28, 28..35],
                None,
            ),
            Ok(Some(-22.5)),
        );

        assert_eq_float!(data[7 * (bases.base(0).unwrap() + 1) + 6], 4.5);
        assert_eq_float!(data[7 * (bases.base(1).unwrap() + 1) + 6], 0.0);
    }

    #[test]
    fn test_simplex_i_p4() {
        let mut bases = [None; 6];

        #[rustfmt::skip]
        let mut data = [
            -5.0, -17.0 / 4.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 0.0, 0.0, 45.0,
            1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 4.0,
            0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 1.0,
        ];

        assert_eq!(
            simplex::<10, ()>(
                &mut bases,
                &mut data,
                &[0..7, 7..14, 14..21, 21..28, 28..35],
                None,
            ),
            Ok(None),
        );
    }
}
