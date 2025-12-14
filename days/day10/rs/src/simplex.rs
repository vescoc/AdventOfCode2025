use core::mem;
use core::ops;

use crate::set::Set;
use crate::LOG;

const EPS: f64 = 1e-6;

#[derive(Debug, PartialEq)]
pub enum Error {
    Partitions,
    Len,
}

/// # Panics
pub fn simplex_eqs<T>(
    bases: &mut (impl Set + core::fmt::Binary),
    eqs: &mut [&mut [f64]],
    tags: &mut [T],
) -> f64 {
    if LOG {
        println!("start");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    reduce(bases, &mut eqs[1..], tags);

    if LOG {
        println!("reduce {bases:016b}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    change_bases(bases, &mut eqs[1..]);

    if LOG {
        println!("change bases {bases:016b}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    {
        let (eqc, eqs) = eqs.split_first_mut().unwrap();
        tableau(bases, eqc, eqs);
    }

    if LOG {
        println!("tableau {bases:016b}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    let len = eqs[0].len();
    while let Some(base_in) = eqs[0].iter().take(len - 1).enumerate().find_map(|(j, v)| {
        if *v < -EPS && !bases.is_set(j) {
            Some(j)
        } else {
            None
        }
    }) {
        if LOG {
            println!("working on {base_in}");
        }
        
        let (i, _) = find(&eqs[1..], base_in).expect("???");
        let base_out = find_base(bases, eqs[i + 1]).expect("cannot find base on {i}");

        assert!(base_in != base_out);

        pivot(eqs, i + 1, base_in);

        bases.reset(base_out);
        bases.set(base_in);

        if LOG {
            println!("pivot {bases:016b} from {base_out} to {base_in}");
            for eq in eqs.iter() {
                println!("{eq:?}");
            }
        }
    }

    if LOG {
        println!("done {bases:016b}");
        for eq in eqs.iter() {
            println!("{eq:?}");
        }
    }

    -eqs[0][len - 1]
}

/// # Errors
pub fn simplex<const SIZE: usize, T>(
    bases: &mut (impl Set + core::fmt::Binary),
    data: &mut [f64],
    partitions: &[ops::Range<usize>],
    tags: &mut [T],
) -> Result<f64, Error> {
    partition::<SIZE, _, _>(data, partitions, |eqs| simplex_eqs(bases, eqs, tags))
}

/// # Errors
pub fn partition<'a, const SIZE: usize, T, U>(
    data: &'a mut [T],
    partitions: &[ops::Range<usize>],
    mut f: impl FnMut(&mut [&'a mut [T]]) -> U,
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
    mut f: impl FnMut(&mut [&'a mut [T]]) -> U,
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

fn find_base(bases: &impl Set, equation: &[f64]) -> Option<usize> {
    equation
        .iter()
        .enumerate()
        .take(equation.len() - 1)
        .find_map(|(i, v)| {
            if bases.is_set(i) && (*v - 1.0).abs() < EPS {
                Some(i)
            } else {
                None
            }
        })
}

fn change_bases(bases: &mut impl Set, equations: &mut [&mut [f64]]) {
    while let Some(((i, new_base), _)) = equations
        .iter()
        .enumerate()
        .filter_map(|(n, eq)| {
            eq.last().and_then(|&b| {
                if b < -EPS {
                    eq.iter()
                        .take(eq.len() - 1)
                        .enumerate()
                        .filter_map(|(i, &v)| {
                            if !bases.is_set(i) && v < -EPS {
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
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    {
        let old_base = find_base(bases, equations[i]).expect("cannot find base on {i}");

        assert!(old_base != new_base);
        
        pivot(equations, i, new_base);

        bases.reset(old_base);
        bases.set(new_base);

        if LOG {
            println!("change base from {old_base} to {new_base}");
        }
    }
}

// #[allow(clippy::needless_range_loop)]
// fn pivot(equations: &mut [&mut [f64]], i: usize, j: usize) {
//     let d = equations[i][j];
//     for e in equations[i].iter_mut() {
//         *e /= d;
//     }

//     let l = equations.len();
//     for k in 0..l {
//         if k != i {
//             let ll = equations[k].len();
//             let mul = equations[k][j];
//             for ii in 0..ll {
//                 equations[k][ii] -= mul * equations[i][ii];
//             }
//         }
//     }
// }

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

fn tableau(bases: &impl Set, eqc: &mut [f64], equations: &[&mut [f64]]) {
    for j in 0..eqc.len() - 1 {
        if bases.is_set(j) {
            let eq = equations
                .iter()
                .find(|eq| (eq[j] - 1.0).abs() < EPS)
                .unwrap();
            let mul = eqc[j];
            for (e, v) in eqc.iter_mut().zip(eq.iter()) {
                *e -= mul * v;
            }
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn reduce<T>(bases: &mut impl Set, equations: &mut [&mut [f64]], tags: &mut [T]) {
    let n = equations.len();
    if n == 0 {
        return;
    }

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

        bases.set(j);

        equations.swap(r, 0);
        tags.swap(r, 0);

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

        equations.swap(0, r);
        tags.swap(0, r);

        equations.swap(r, i);
        tags.swap(r, i);

        i += 1;
        j += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reduce_1() {
        let mut eq1 = [1.0, 1.0, 2.0];
        let mut eq2 = [1.0, -1.0, 0.0];
        let mut eqs = [eq1.as_mut_slice(), eq2.as_mut_slice()];

        let mut tags = [0, 1];
        let mut bases = 0u16;
        reduce(&mut bases, &mut eqs, &mut tags);

        assert_eq!(
            eqs,
            [[1.0, 0.0, 1.0].as_slice(), [0.0, 1.0, 1.0].as_slice()]
        );
        assert_eq!(bases, 0b11);
    }

    #[test]
    fn test_reduce_2() {
        let mut eq1 = [0.0, 2.0, 2.0];
        let mut eq2 = [2.0, 0.0, 2.0];
        let mut eqs = [eq1.as_mut_slice(), eq2.as_mut_slice()];

        let mut tags = [0, 1];
        let mut bases = 0u16;
        reduce(&mut bases, &mut eqs, &mut tags);

        assert_eq!(
            eqs,
            [[1.0, 0.0, 1.0].as_slice(), [0.0, 1.0, 1.0].as_slice()]
        );
        assert_eq!(bases, 0b11);
    }

    #[test]
    fn test_reduce_3() {
        let mut eq1 = [1.0, 1.0, 1.0, 3.0];
        let mut eq2 = [1.0, -1.0, 0.0, 0.0];
        let mut eq3 = [0.0, 1.0, 1.0, 2.0];
        let mut eqs = [eq1.as_mut_slice(), eq2.as_mut_slice(), eq3.as_mut_slice()];

        let mut tags = [0, 1, 2];
        let mut bases = 0u16;
        reduce(&mut bases, &mut eqs, &mut tags);

        assert_eq!(
            eqs,
            [
                [1.0, 0.0, 0.0, 1.0].as_slice(),
                [0.0, 1.0, 0.0, 1.0].as_slice(),
                [0.0, 0.0, 1.0, 1.0].as_slice()
            ]
        );
        assert_eq!(bases, 0b111);
    }

    #[test]
    fn test_simplex_4() {
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
        let mut bases = 0u16;
        assert_eq!(
            simplex::<10, _>(
                &mut bases,
                &mut data,
                &[0..5, 5..10, 10..15, 15..20, 20..25, 25..30, 30..35,],
                &mut tags,
            ),
            Ok(43.0),
        );
    }

    #[test]
    fn test_reduce_4() {
        let mut eq0 = [0.0, 1.0, 0.0, 0.0, 5.0];
        let mut eq1 = [1.0, 0.0, 1.0, 1.0, 38.0];
        let mut eq2 = [1.0, 0.0, 0.0, 0.0, 18.0];
        let mut eq3 = [0.0, 0.0, 1.0, 1.0, 20.0];
        let mut eq4 = [0.0, 1.0, 0.0, 0.0, 5.0];
        let mut eq5 = [0.0, 0.0, 0.0, 1.0, 14.0];
        let mut eqs = [
            eq0.as_mut_slice(),
            eq1.as_mut_slice(),
            eq2.as_mut_slice(),
            eq3.as_mut_slice(),
            eq4.as_mut_slice(),
            eq5.as_mut_slice(),
        ];

        let mut tags = [0, 1, 2, 3, 4, 5];
        let mut bases = 0u16;
        reduce(&mut bases, &mut eqs, &mut tags);

        assert_eq!(bases.size(), 4);

        assert_eq!(
            eqs,
            [
                &[1.0, 0.0, 0.0, 0.0, 18.0],
                &[0.0, 1.0, 0.0, 0.0, 5.0],
                &[0.0, 0.0, 1.0, 0.0, 6.0],
                &[0.0, 0.0, 0.0, 1.0, 14.0],
                &[0.0, 0.0, 0.0, 0.0, 0.0],
                &[0.0, 0.0, 0.0, 0.0, 0.0],
            ]
        );
        assert_eq!(bases, 0b1111,);
    }

    // [.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_reduce_5() {
        let mut eq0 = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 3.0];
        let mut eq1 = [0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 5.0];
        let mut eq2 = [0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 4.0];
        let mut eq3 = [1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0];
        let mut eqs = [
            eq0.as_mut_slice(),
            eq1.as_mut_slice(),
            eq2.as_mut_slice(),
            eq3.as_mut_slice(),
        ];

        let mut bases = 0u16;
        let mut tags = [0, 1, 2, 3];
        reduce(&mut bases, &mut eqs, &mut tags);

        assert_eq!(bases.size(), 4);

        assert_eq!(
            eqs,
            [
                &[1.0, 0.0, 0.0, 1.0, 0.0, -1.0, 2.0],
                &[0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 5.0],
                &[0.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0],
                &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 3.0],
            ]
        );

        assert_eq!(bases, 0b010111);
    }

    #[test]
    fn test_simplex() {
        let mut bases = 0u16;

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
                &mut tags,
            ),
            Ok(10.0),
        );
    }

    #[test]
    fn test_simplex_5() {
        let mut eqc = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0];
        let mut eq0 = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 3.0];
        let mut eq1 = [0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 5.0];
        let mut eq2 = [0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 4.0];
        let mut eq3 = [1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0];

        let mut bases = {
            let mut eqs = [
                eq0.as_mut_slice(),
                eq1.as_mut_slice(),
                eq2.as_mut_slice(),
                eq3.as_mut_slice(),
            ];

            let mut bases = 0u16;
            let mut tags = [0, 1, 2, 3, 4];
            reduce(&mut bases, &mut eqs, &mut tags);

            bases
        };

        {
            let eqs = [
                eq0.as_mut_slice(),
                eq1.as_mut_slice(),
                eq2.as_mut_slice(),
                eq3.as_mut_slice(),
            ];
            tableau(&bases, &mut eqc, &eqs);
        }

        while let Some(j) = eqc
            .iter()
            .take(eqc.len() - 1)
            .enumerate()
            .find_map(|(j, v)| {
                if *v < 0.0 && !bases.is_set(j) {
                    Some(j)
                } else {
                    None
                }
            })
        {
            let Some((i, _)) = ({
                let eqs = [
                    eq0.as_mut_slice(),
                    eq1.as_mut_slice(),
                    eq2.as_mut_slice(),
                    eq3.as_mut_slice(),
                ];
                find(&eqs, j)
            }) else {
                unreachable!("???");
            };

            {
                let mut eqs = [
                    eqc.as_mut_slice(),
                    eq0.as_mut_slice(),
                    eq1.as_mut_slice(),
                    eq2.as_mut_slice(),
                    eq3.as_mut_slice(),
                ];
                pivot(&mut eqs, i + 1, j);
            }

            bases.set(j);
            bases.reset(i);
        }

        assert!((-eqc[eqc.len() - 1] - 10.0).abs() < EPS);
    }

    // [...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
    #[test]
    fn test_reduce_6() {
        let mut eq0 = [1.0, 0.0, 1.0, 1.0, 0.0, 7.0];
        let mut eq1 = [0.0, 0.0, 0.0, 1.0, 1.0, 5.0];
        let mut eq2 = [1.0, 1.0, 0.0, 1.0, 1.0, 12.0];
        let mut eq3 = [1.0, 1.0, 0.0, 0.0, 1.0, 7.0];
        let mut eq4 = [1.0, 0.0, 1.0, 0.0, 1.0, 2.0];
        let mut eqs = [
            eq0.as_mut_slice(),
            eq1.as_mut_slice(),
            eq2.as_mut_slice(),
            eq3.as_mut_slice(),
            eq4.as_mut_slice(),
        ];

        let mut bases = 0u16;
        let mut tags = [0, 1, 2, 3, 4];
        reduce(&mut bases, &mut eqs, &mut tags);

        assert_eq!(bases.size(), 4);

        assert_eq!(
            eqs,
            [
                &[1.0, 0.0, 1.0, 0.0, 0.0, 2.0],
                &[0.0, 1.0, -1.0, 0.0, 0.0, 5.0],
                &[0.0, 0.0, 0.0, 1.0, 0.0, 5.0],
                &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
        );

        assert_eq!(bases, 0b11011);
    }

    #[test]
    fn test_simplex_6() {
        let mut bases = 0u16;

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
                &mut tags,
            ),
            Ok(12.0),
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
    fn test_simplex_xxx() {
        let mut eqc = [-130.0, -100.0, 0.0, 0.0, 0.0, 0.0];

        let mut eq0 = [1.5, 1.0, 1.0, 0.0, 0.0, 27.0];
        let mut eq1 = [1.0, 1.0, 0.0, 1.0, 0.0, 21.0];
        let mut eq2 = [0.3, 0.5, 0.0, 0.0, 1.0, 9.0];

        assert!(eqc[0] < 0.0);
        assert!(eqc[1] < 0.0);

        let i = {
            let eqs = [eq0.as_mut_slice(), eq1.as_mut_slice(), eq2.as_mut_slice()];

            find(&eqs, 0).unwrap().0
        };

        {
            let mut eqs = [
                eqc.as_mut_slice(),
                eq0.as_mut_slice(),
                eq1.as_mut_slice(),
                eq2.as_mut_slice(),
            ];

            pivot(&mut eqs, i + 1, 0);
        }

        assert!(eqc[1] < 0.0);

        let i = {
            let eqs = [eq0.as_mut_slice(), eq1.as_mut_slice(), eq2.as_mut_slice()];

            find(&eqs, 1).unwrap().0
        };

        {
            let mut eqs = [
                eqc.as_mut_slice(),
                eq0.as_mut_slice(),
                eq1.as_mut_slice(),
                eq2.as_mut_slice(),
            ];

            pivot(&mut eqs, i + 1, 1);
        }

        {
            let target = [
                &[0.0, 0.0, 60.0, 40.0, 0.0, 2460.0],
                &[1.0, 0.0, 2.0, -2.0, 0.0, 12.0],
                &[0.0, 1.0, -2.0, 3.0, 0.0, 9.0],
                &[0.0, 0.0, 0.4, -0.9, 1.0, 0.9],
            ];

            let eqs = [
                eqc.as_mut_slice(),
                eq0.as_mut_slice(),
                eq1.as_mut_slice(),
                eq2.as_mut_slice(),
            ];

            for (i, (a, b)) in eqs.iter().zip(target.iter()).enumerate() {
                for (j, (a, b)) in a.iter().zip(b.iter()).enumerate() {
                    assert!((a - b).abs() < 1e-6, "{:?}: {a} != {b}", (i, j));
                }
            }
        }
    }

    #[test]
    fn test_simplex_ex() {
        let mut bases = 0u16;

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
                &mut tags
            ),
            Ok(-2460.0),
        );
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
        let mut bases = 0u16;

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
                &mut tags,
            ),
            Ok(11.0),
        );
    }

    #[test]
    fn test_partitions() {
        #[rustfmt::skip]
        let mut data = [1.0, 1.0, 2.0,
                        1.0, -1.0, 0.0];

        let mut bases = 0u16;
        let mut tags = [0, 1];
        partition::<2, _, _>(&mut data, &[0..3, 3..6], |eqs| {
            reduce(&mut bases, eqs, &mut tags);
        })
        .unwrap();

        assert!(
            data.iter()
                .zip(&[1.0, 0.0, 1.0, 0.0, 1.0, 1.0])
                .all(|(a, b)| (a - b).abs() < EPS)
        );
    }
}
