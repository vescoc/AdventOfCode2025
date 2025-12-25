#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_float_math))]

use core::ops;

use slice_partitions::{Error as PartitionError, partition};

pub(crate) mod sealed {
    pub trait Sealed {}
}

pub trait StrategyType: sealed::Sealed {
    fn prune<F: Float>(solution: F, incumbent_solution: F) -> bool;
    fn is_better_of<F: Float>(a: F, b: F) -> bool;
}

#[derive(Debug)]
pub struct Min;

#[derive(Debug)]
pub struct Max;

impl sealed::Sealed for Min {}
impl sealed::Sealed for Max {}

impl StrategyType for Min {
    #[inline]
    fn prune<F: Float>(solution: F, incumbent_solution: F) -> bool {
        solution >= incumbent_solution
    }

    #[inline]
    fn is_better_of<F: Float>(a: F, b: F) -> bool {
        a < b
    }
}

impl StrategyType for Max {
    #[inline]
    fn prune<F: Float>(solution: F, incumbent_solution: F) -> bool {
        solution <= incumbent_solution
    }

    #[inline]
    fn is_better_of<F: Float>(a: F, b: F) -> bool {
        a < b
    }
}

pub trait Float
where
    Self: Copy,
    Self: core::cmp::PartialOrd,
    Self: ops::Sub<Output = Self>
        + ops::Div<Output = Self>
        + ops::Mul<Output = Self>
        + ops::Add<Output = Self>,
    Self: ops::DivAssign + ops::SubAssign,
    Self: ops::Neg<Output = Self>,
{
    const EPS: Self;
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const ONE: Self;
    const TEN: Self;

    fn f_as_u64(self) -> u64;

    #[must_use]
    fn abs(self) -> Self;

    #[must_use]
    fn min(self, other: Self) -> Self;

    #[must_use]
    fn max(self, other: Self) -> Self;

    #[must_use]
    fn f_round(self) -> Self;

    #[must_use]
    fn f_ceil(self) -> Self;

    #[must_use]
    fn f_floor(self) -> Self;
}

macro_rules! impl_float {
    ($t:tt, $eps:literal) => {
        impl Float for $t {
            const EPS: Self = $eps;
            const MAX: Self = <$t>::MAX;
            const MIN: Self = <$t>::MIN;
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const TEN: Self = 10.0;

            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            fn f_as_u64(self) -> u64 {
                self as u64
            }

            #[inline]
            fn abs(self) -> Self {
                <$t>::abs(self)
            }

            #[inline]
            fn min(self, other: Self) -> Self {
                <$t>::min(self, other)
            }

            #[inline]
            fn max(self, other: Self) -> Self {
                <$t>::max(self, other)
            }

            #[inline]
            #[cfg(feature = "std")]
            fn f_round(self) -> Self {
                <$t>::round(self)
            }

            #[inline]
            #[cfg(feature = "std")]
            fn f_ceil(self) -> Self {
                <$t>::ceil(self)
            }

            #[inline]
            #[cfg(feature = "std")]
            fn f_floor(self) -> Self {
                <$t>::floor(self)
            }

            #[inline]
            #[cfg(not(feature = "std"))]
            fn f_round(self) -> Self {
                impl_float!(@ $t round self)
            }

            #[inline]
            #[cfg(not(feature = "std"))]
            fn f_ceil(self) -> Self {
                impl_float!(@ $t ceil self)
            }

            #[inline]
            #[cfg(not(feature = "std"))]
            fn f_floor(self) -> Self {
                impl_float!(@ $t floor self)
            }
        }
    };

    (@ f64 round $self:ident) => { core::f64::math::round($self) };
    (@ f64 ceil $self:ident) => { core::f64::math::ceil($self) };
    (@ f64 floor $self:ident) => { core::f64::math::floor($self) };

    (@ f32 round $self:ident) => { core::f32::math::round($self) };
    (@ f32 ceil $self:ident) => { core::f32::math::ceil($self) };
    (@ f32 floor $self:ident) => { core::f32::math::floor($self) };
}

impl_float!(f64, 1e-6);
impl_float!(f32, 1e-3);

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

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Out of memory")]
    Oom,

    #[error("Invalid data")]
    InvalidData,
}

/// # Errors
/// # Panics
#[allow(clippy::too_many_lines)]
pub fn integer_simplex<const BASES: usize, const SCRATCH_SIZE: usize, F: Float, S: StrategyType>(
    _strategy: S,
    xi: &mut [Option<u64>],
    zi: &[F],
    aij: &[F],
    bj: &[F],
) -> Result<Option<F>, Error> {
    let zi_len = zi.len();
    let aij_len = aij.len();
    let bj_len = bj.len();

    if !(aij_len == zi_len * bj_len && zi_len > 0 && bj_len > 0 && xi.len() == zi_len) {
        return Err(Error::InvalidData);
    }

    let mut incumbent_solution = None;

    let mut stack = heapless::Vec::<_, BASES>::new();
    stack
        .push(heapless::Vec::<_, BASES>::new())
        .map_err(|_| Error::Oom)?;
    while let Some(partitions) = stack.pop() {
        // allocate all the data, initialized to 0.0
        let cols = zi_len + 1 + partitions.len();
        let rows = bj_len + 1 + partitions.len();
        let data_len = cols * rows;

        let mut data = [F::ZERO; SCRATCH_SIZE];
        if data_len >= data.len() {
            return Err(Error::Oom);
        }

        // init the max xi zj part
        data[..zi_len].copy_from_slice(zi);

        // init the aij part
        for (data, (ai, b)) in data[..data_len]
            .chunks_exact_mut(cols)
            .skip(1)
            .zip(aij.chunks_exact(zi_len).zip(bj))
        {
            data[..zi_len].copy_from_slice(ai);
            data[cols - 1] = *b;
        }

        // init the partitions part
        for (data, (i, (x, sign, value))) in data[..data_len]
            .chunks_exact_mut(cols)
            .skip(bj_len + 1)
            .zip(partitions.iter().enumerate())
        {
            data[*x] = F::ONE;
            data[zi_len + i] = *sign;
            data[cols - 1] = *value;
        }

        let mut eqs = heapless::Vec::<_, BASES>::new();
        for eq in data[..data_len].chunks_exact_mut(cols) {
            eqs.push(eq).map_err(|_| Error::Oom)?;
        }

        let mut bases = [None; BASES];
        let Some(solution) = simplex_eqs(&mut bases, &mut eqs, None::<&mut [()]>) else {
            // pruned by infeasibility
            continue;
        };

        let check_i = bases.iter().enumerate().find_map(|(x, n)| {
            n.and_then(|n| {
                let v = eqs[n + 1].last().unwrap();
                if (v.f_round() - *v).abs() >= F::EPS {
                    Some((x, v))
                } else {
                    None
                }
            })
        });

        if let Some((x, v)) = check_i {
            if let Some(incumbent_solution) = incumbent_solution
                && S::prune(solution, incumbent_solution)
            {
                // prune by bound
                continue;
            }

            {
                let mut found = false;
                let mut new_partitions = partitions.clone();
                if let Some((_, s, vv)) = new_partitions.iter_mut().find(|(xx, _, _)| *xx == x) {
                    *s = F::ONE;
                    *vv = v.f_floor();
                    found = true;
                } else {
                    new_partitions
                        .push((x, F::ONE, v.f_floor()))
                        .map_err(|_| Error::Oom)?;
                }

                if (!found || check_partitions(&new_partitions, &partitions))
                    && stack.iter().all(|p| check_partitions(&new_partitions, p))
                {
                    stack.push(new_partitions).map_err(|_| Error::Oom)?;
                }
            }

            {
                let mut found = false;
                let mut new_partitions = partitions.clone();
                if let Some((_, s, vv)) = new_partitions.iter_mut().find(|(xx, _, _)| *xx == x) {
                    *s = -F::ONE;
                    *vv = v.f_ceil();
                    found = true;
                } else {
                    new_partitions
                        .push((x, -F::ONE, v.f_ceil()))
                        .map_err(|_| Error::Oom)?;
                }

                if (!found || check_partitions(&new_partitions, &partitions))
                    && stack.iter().all(|p| check_partitions(p, &new_partitions))
                {
                    stack.push(new_partitions).map_err(|_| Error::Oom)?;
                }
            }
        } else if let Some(incumbent_solution) = &mut incumbent_solution {
            if S::is_better_of(solution, *incumbent_solution) {
                *incumbent_solution = solution;

                for (x, n) in xi.iter_mut().zip(bases.iter()) {
                    *x = n.map(|n| eqs[n + 1].last().unwrap().f_as_u64());
                }
            }
        } else {
            incumbent_solution.replace(solution.f_round());

            for (x, n) in xi.iter_mut().zip(bases.iter()) {
                *x = n.map(|n| eqs[n + 1].last().unwrap().f_as_u64());
            }
        }
    }

    Ok(incumbent_solution)
}

fn check_partitions<F: Float>(a: &[(usize, F, F)], b: &[(usize, F, F)]) -> bool {
    a.len() != b.len()
        || a.iter()
            .zip(b)
            .any(|(a, b)| a.0 != b.0 || (a.1 - b.1).abs() > F::EPS || (a.2 - b.2).abs() > F::EPS)
}

/// # Panics
#[allow(clippy::similar_names)]
pub fn simplex_eqs<T, F: Float>(
    bases: &mut impl BaseMap,
    eqs: &mut [&mut [F]],
    tags: Option<&mut [T]>,
) -> Option<F> {
    bases.reset();

    if !reduce(bases, &mut eqs[1..], tags) {
        return None;
    }

    if !change_bases(bases, &mut eqs[1..]) {
        return None;
    }

    {
        let (eqc, eqs) = eqs.split_first_mut().unwrap();
        tableau(bases, eqc, eqs);
    }

    let len = eqs[0].len();
    while let Some(base_in) = eqs[0].iter().take(len - 1).enumerate().find_map(|(j, v)| {
        if *v < -F::EPS && !bases.is_base(j) {
            Some(j)
        } else {
            None
        }
    }) {
        let (i, _) = find(&eqs[1..], base_in).expect("???");
        let base_out = bases.base_for_equation(i).expect("cannot find base on {i}");

        assert!(base_in != base_out);

        pivot(eqs, i + 1, base_in);

        bases.base_out(base_out);
        bases.base_in(base_in, i);
    }

    Some(-eqs[0][len - 1])
}

/// # Errors
pub fn simplex<const SIZE: usize, T, F: Float>(
    bases: &mut impl BaseMap,
    data: &mut [F],
    partitions: &[ops::Range<usize>],
    tags: Option<&mut [T]>,
) -> Result<Option<F>, PartitionError> {
    partition::<SIZE, _, _>(data, partitions, move |eqs| simplex_eqs(bases, eqs, tags))
}

fn change_bases<F: Float>(bases: &mut impl BaseMap, equations: &mut [&mut [F]]) -> bool {
    while let Some(((i, new_base), _)) = equations
        .iter()
        .enumerate()
        .filter_map(|(n, eq)| {
            bases.base_for_equation(n).and_then(|_| {
                eq.last().and_then(|&b| {
                    if b < -F::EPS {
                        eq.iter()
                            .take(eq.len() - 1)
                            .enumerate()
                            .filter_map(|(i, &v)| {
                                if !bases.is_base(i) && v < -F::EPS {
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
    }

    equations
        .iter()
        .enumerate()
        .all(|(n, eq)| bases.base_for_equation(n).is_none() || eq[eq.len() - 1] > -F::EPS)
}

fn pivot<F: Float>(equations: &mut [&mut [F]], i: usize, j: usize) {
    equations.swap(i, 0);

    let (first, eqs) = equations.split_first_mut().unwrap();

    let d = first[j];
    for e in first.iter_mut() {
        *e /= d;
    }

    for eq in eqs.iter_mut() {
        let mul = eq[j];
        for (e, v) in eq.iter_mut().zip(first.iter()) {
            *e -= mul * *v;
        }
    }

    equations.swap(i, 0);
}

fn find<F: Float>(equations: &[&mut [F]], k: usize) -> Option<(usize, F)> {
    let l = equations[0].len();
    equations
        .iter()
        .enumerate()
        .filter_map(|(i, eq)| {
            let aik = eq[k];
            if aik > F::EPS {
                Some((i, eq[l - 1] / aik))
            } else {
                None
            }
        })
        .min_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap())
}

fn tableau<F: Float>(bases: &impl BaseMap, eqc: &mut [F], equations: &[&mut [F]]) {
    for j in 0..eqc.len() - 1 {
        if let Some(eq) = bases.base(j).and_then(|k| equations.get(k)) {
            let mul = eqc[j];
            for (e, v) in eqc.iter_mut().zip(eq.iter()) {
                *e -= mul * *v;
            }
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn reduce<T, F: Float>(
    bases: &mut impl BaseMap,
    equations: &mut [&mut [F]],
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
            .position(|(candidate, equation)| candidate >= i && equation[j] != F::ZERO)
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
                *e -= *v * mul;
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
        .all(|eq| eq[k - 1].abs() <= F::EPS || eq.iter().take(k - 1).any(|v| v.abs() >= F::EPS))
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_eq_f64 {
        ($e1:expr, $e2:expr) => {
            assert!(($e1 - $e2).abs() < f64::EPS, "{} != {}", $e1, $e2);
        };
    }

    macro_rules! assert_eq_f32 {
        ($e1:expr, $e2:expr) => {
            assert!(($e1 - $e2).abs() < f32::EPS, "{} != {}", $e1, $e2);
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
            simplex::<10, _, _>(
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
            simplex::<10, _, _>(
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
            simplex::<10, _, _>(
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
            simplex::<10, _, _>(
                &mut bases,
                &mut data,
                &[0..6, 6..12, 12..18, 18..24,],
                Some(&mut tags),
            ),
            Ok(Some(-2460.0)),
        );

        assert!((data[6 * (bases.base(0).unwrap() + 1) + 5] - 12.0).abs() < f64::EPS);
        assert!((data[6 * (bases.base(1).unwrap() + 1) + 5] - 9.0).abs() < f64::EPS);
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
            simplex::<10, _, _>(
                &mut bases,
                &mut data,
                &[0..5, 5..10, 10..15, 15..20, 20..25, 25..30, 30..35,],
                Some(&mut tags),
            ),
            Ok(Some(11.0)),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_simplex_i_f64() {
        let mut xi = [None; 4];
        let zi = [-5.0, -17.0 / 4.0, 0.0, 0.0];
        let aij = [1.0, 1.0, 1.0, 0.0,
                   10.0, 6.0, 0.0, 1.0];
        let bj = [5.0, 45.0];
        assert_eq!(
            integer_simplex::<10, 100, f64, _>(Max, &mut xi, &zi, &aij, &bj),
            Ok(Some(-23.5)),
        );

        assert_eq!(xi[0], Some(3));
        assert_eq!(xi[1], Some(2));
    }

    #[test]
    #[rustfmt::skip]
    fn test_simplex_i_f32() {
        let mut xi = [None; 4];
        let zi = [-5.0, -17.0 / 4.0, 0.0, 0.0];
        let aij = [1.0, 1.0, 1.0, 0.0,
                   10.0, 6.0, 0.0, 1.0];
        let bj = [5.0, 45.0];
        assert_eq!(
            integer_simplex::<10, 100, f32, _>(Max, &mut xi, &zi, &aij, &bj),
            Ok(Some(-23.5)),
        );

        assert_eq!(xi[0], Some(3));
        assert_eq!(xi[1], Some(2));
    }

    #[test]
    fn test_simplex_i_p0() {
        let mut bases = [None; 4];

        #[rustfmt::skip]
        let mut data = [
            -5.0f64, -17.0 / 4.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 45.0,
        ];

        assert_eq!(
            simplex::<10, (), _>(&mut bases, &mut data, &[0..5, 5..10, 10..15], None,),
            Ok(Some(-24.0625)),
        );

        assert_eq_f64!(data[5 * (bases.base(0).unwrap() + 1) + 4], 3.75);
        assert_eq_f64!(data[5 * (bases.base(1).unwrap() + 1) + 4], 1.25);
    }

    #[test]
    fn test_simplex_i_p0_f32() {
        let mut bases = [None; 4];

        #[rustfmt::skip]
        let mut data = [
            -5.0f32, -17.0 / 4.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 45.0,
        ];

        assert_eq!(
            simplex::<10, (), _>(&mut bases, &mut data, &[0..5, 5..10, 10..15], None,),
            Ok(Some(-24.0625)),
        );

        assert_eq_f32!(data[5 * (bases.base(0).unwrap() + 1) + 4], 3.75);
        assert_eq_f32!(data[5 * (bases.base(1).unwrap() + 1) + 4], 1.25);
    }

    #[test]
    fn test_simplex_i_p1() {
        let mut bases = [None; 5];

        #[rustfmt::skip]
        let mut data = [
            -5.0f64, -17.0 / 4.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 0.0, 45.0,
            1.0, 0.0, 0.0, 0.0, 1.0, 3.0,
        ];

        assert_eq!(
            simplex::<10, (), _>(&mut bases, &mut data, &[0..6, 6..12, 12..18, 18..24], None,),
            Ok(Some(-23.5)),
        );

        assert_eq_f64!(data[6 * (bases.base(0).unwrap() + 1) + 5], 3.0);
        assert_eq_f64!(data[6 * (bases.base(1).unwrap() + 1) + 5], 2.0);
    }

    #[test]
    fn test_simplex_i_p1_f32() {
        let mut bases = [None; 5];

        #[rustfmt::skip]
        let mut data = [
            -5.0f32, -17.0 / 4.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 5.0,
            10.0, 6.0, 0.0, 1.0, 0.0, 45.0,
            1.0, 0.0, 0.0, 0.0, 1.0, 3.0,
        ];

        assert_eq!(
            simplex::<10, (), _>(&mut bases, &mut data, &[0..6, 6..12, 12..18, 18..24], None,),
            Ok(Some(-23.5)),
        );

        assert_eq_f32!(data[6 * (bases.base(0).unwrap() + 1) + 5], 3.0);
        assert_eq_f32!(data[6 * (bases.base(1).unwrap() + 1) + 5], 2.0);
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
            simplex::<10, (), _>(&mut bases, &mut data, &[0..6, 6..12, 12..18, 18..24], None,),
            Ok(Some(-23.541666666666668)),
        );

        assert_eq_f64!(data[6 * (bases.base(0).unwrap() + 1) + 5], 4.0);
        assert_eq_f64!(
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
            simplex::<10, (), _>(
                &mut bases,
                &mut data,
                &[0..7, 7..14, 14..21, 21..28, 28..35],
                None,
            ),
            Ok(Some(-22.5)),
        );

        assert_eq_f64!(data[7 * (bases.base(0).unwrap() + 1) + 6], 4.5);
        assert_eq_f64!(data[7 * (bases.base(1).unwrap() + 1) + 6], 0.0);
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
            simplex::<10, (), _>(
                &mut bases,
                &mut data,
                &[0..7, 7..14, 14..21, 21..28, 28..35],
                None,
            ),
            Ok(None),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_integer_simplex() {
        let mut xi = [None; 10];
        let zi = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let aij = [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0];
        let bj = [59.0, 23.0, 42.0, 27.0, 39.0, 21.0, 40.0, 32.0];
        
        #[rustfmt::skip]
        assert_eq!(
            integer_simplex::<32, { 32 * 32 }, _, _>(
                Min,
                &mut xi,
                &zi,
                &aij,
                &bj,
            ),
            Ok(Some(82.0)),
        );
    }

    #[test]
    #[rustfmt::skip]
    fn test_integer_simplex_f32() {
        let mut xi = [None; 10];
        let zi = [1.0f32, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let aij = [0.0f32, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0];
        let bj = [59.0f32, 23.0, 42.0, 27.0, 39.0, 21.0, 40.0, 32.0];
        
        assert_eq!(
            integer_simplex::<32, { 32 * 32 }, _, _>(
                Min,
                &mut xi,
                &zi,
                &aij,
                &bj,
            ),
            Ok(Some(81.99999)),
        );
    }

    #[test]
    fn test_integer_simplex_1_f64() {
        let mut xi = [None; 10];
        let zi = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let aij = [
            0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        ];
        let bj = [50.0, 69.0, 17.0, 60.0, 52.0, 62.0, 66.0, 42.0];

        assert_eq!(
            integer_simplex::<32, { 32 * 32 }, f64, _>(Min, &mut xi, &zi, &aij, &bj,),
            Ok(Some(106.0)),
        );
    }

    #[test]
    fn test_integer_simplex_1_f32() {
        let mut xi = [None; 10];
        let zi = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let aij = [
            0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        ];
        let bj = [50.0, 69.0, 17.0, 60.0, 52.0, 62.0, 66.0, 42.0];

        assert_eq!(
            integer_simplex::<32, { 32 * 32 }, f32, _>(Min, &mut xi, &zi, &aij, &bj,),
            Ok(Some(106.0)),
        );
    }

    #[test]
    fn test_integer_simplex_2_f64() {
        let mut xi = [None; 9];
        let zi = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let aij = [
            0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];
        let bj = [16.0, 22.0, 32.0, 46.0, 9.0, 27.0, 49.0];

        assert_eq!(
            integer_simplex::<32, { 32 * 32 }, f64, _>(Min, &mut xi, &zi, &aij, &bj,),
            Ok(Some(54.0)),
        );
    }

    #[test]
    fn test_integer_simplex_2_f32() {
        let mut xi = [None; 9];
        let zi = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let aij = [
            0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
            0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];
        let bj = [16.0, 22.0, 32.0, 46.0, 9.0, 27.0, 49.0];

        assert_eq!(
            integer_simplex::<32, { 32 * 32 }, f32, _>(Min, &mut xi, &zi, &aij, &bj,),
            Ok(Some(54.0)),
        );
    }
}
