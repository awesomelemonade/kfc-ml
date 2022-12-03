use std::{cmp::Ordering, mem};

use itertools::Itertools;
use pyo3::{PyResult, Python};
use rayon::prelude::*;

#[cfg(test)]
mod util_tests;

pub fn parallel_map_prioritized_by<'a: 'b, 'b, T: 'b, U, V, F, P>(
    vec: &'a [T],
    map_op: F,
    prioritizer: P,
) -> Vec<U>
where
    &'b T: Send,
    U: Send,
    F: Fn(&T) -> U + Sync,
    P: Fn(&T) -> V,
    V: Ord,
{
    let unsorted: Vec<_> = vec
        .iter()
        .enumerate()
        .sorted_by_cached_key(|(_index, item)| prioritizer(item))
        .par_bridge()
        .map(|(index, item)| (index, map_op(item)))
        .collect();
    unsorted
        .into_iter()
        .sorted_by_key(|(index, _item)| *index)
        .map(|(_index, item)| item)
        .collect_vec()
}

pub trait UnwrapWithTraceback<T> {
    fn unwrap_with_traceback(self, py: Python) -> T;
}

impl<T> UnwrapWithTraceback<T> for PyResult<T> {
    // panics with traceback
    fn unwrap_with_traceback(self, py: Python) -> T {
        match self {
            Ok(t) => t,
            Err(error) => {
                if let Some(traceback) = error.traceback(py) {
                    println!("Traceback:\n{}", traceback.format().unwrap());
                }
                panic!("Unwrap error: {}\n", error);
            }
        }
    }
}

// throws if there are any NaNs
pub fn sort_by_cached_f32_exn<T, F>(vec: &mut Vec<T>, f: F)
where
    F: Fn(&T) -> f32,
{
    sort_by_cached_key_with_comparator(vec, f, |a, b| a.partial_cmp(b).unwrap())
}

// based on https://doc.rust-lang.org/nightly/src/alloc/slice.rs.html#347-350
pub fn sort_by_cached_key_with_comparator<T, F, C, K>(vec: &mut Vec<T>, f: F, comparator: C)
where
    F: Fn(&T) -> K,
    C: Fn(&K, &K) -> Ordering,
{
    // Helper macro for indexing our vector by the smallest possible type, to reduce allocation.
    macro_rules! sort_by_key {
        ($t:ty, $slice:ident, $f:ident, $comparator:ident) => {{
            let mut indices: Vec<_> = $slice
                .iter()
                .map($f)
                .enumerate()
                .map(|(i, k)| (k, i as $t))
                .collect();
            // The elements of `indices` are unique, as they are indexed, so any sort will be
            // stable with respect to the original slice. We use `sort_unstable` here because
            // it requires less memory allocation.
            indices.sort_unstable_by(|(a, a_i), (b, b_i)| {
                let compare = $comparator(a, b);
                if compare == Ordering::Equal {
                    a_i.cmp(b_i)
                } else {
                    compare
                }
            });
            for i in 0..$slice.len() {
                let mut index = indices[i].1;
                while (index as usize) < i {
                    index = indices[index as usize].1;
                }
                indices[i].1 = index;
                $slice.swap(i, index as usize);
            }
        }};
    }

    let sz_u8 = mem::size_of::<(K, u8)>();
    let sz_u16 = mem::size_of::<(K, u16)>();
    let sz_u32 = mem::size_of::<(K, u32)>();
    let sz_usize = mem::size_of::<(K, usize)>();

    let len = vec.len();
    if len < 2 {
        return;
    }
    if sz_u8 < sz_u16 && len <= (u8::MAX as usize) {
        return sort_by_key!(u8, vec, f, comparator);
    }
    if sz_u16 < sz_u32 && len <= (u16::MAX as usize) {
        return sort_by_key!(u16, vec, f, comparator);
    }
    if sz_u32 < sz_usize && len <= (u32::MAX as usize) {
        return sort_by_key!(u32, vec, f, comparator);
    }
    sort_by_key!(usize, vec, f, comparator)
}

// TODO: tests for all these util functions
