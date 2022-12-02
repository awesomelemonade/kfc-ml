core!();

use super::*;

#[test]
pub fn test_parallel_map_retain_order() {
    let vec = vec![5, 2, 3, 1, 4];
    let mapped = parallel_map_prioritized_by(&vec, |x| x + 5, |x| *x);
    expect!(
        mapped,
        r#"
        [
            10,
            7,
            8,
            6,
            9,
        ]"#
    );
}

#[test]
pub fn test_sort_by_cached_f32_exn() {
    let mut vec = vec![0, 1, 2, 3, 4];
    let sort_by = vec![0.3, 0.5, 0.2, 0.4, 0.1];
    sort_by_cached_f32_exn(&mut vec, |i| sort_by[*i]);
    expect!(
        vec,
        r#"
        [
            4,
            2,
            0,
            3,
            1,
        ]"#
    );
}
