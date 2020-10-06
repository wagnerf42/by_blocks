use by_blocks::prelude::*;
use fast_tracer::svg;
use rayon::prelude::*;

fn main() {
    println!("try me with two threads");
    svg("blocked_find.svg", || {
        let start = std::time::Instant::now();
        let powers = std::iter::successors(Some(100_000usize), |s| Some(s.saturating_mul(2)));
        assert_eq!(
            (0..100_000_000)
                .into_par_iter()
                .by_blocks(powers)
                .find_first(|x| *x == 49_900_000),
            Some(49_900_000)
        );
        println!("by blocks: {:?}", start.elapsed());
    })
    .expect("failed saving");
    let start = std::time::Instant::now();
    assert_eq!(
        (0..100_000_000)
            .into_par_iter()
            .find_first(|x| *x == 49_900_000),
        Some(49_900_000)
    );
    println!("rayon: {:?}", start.elapsed());
    println!("********* filter collect ********");
    let v2 = svg("blocked_filter_collect.svg", || {
        let start = std::time::Instant::now();
        let v2: Vec<i32> = (0..16_000_000)
            .into_par_iter()
            .by_blocks_iter(std::iter::repeat(1_000_000))
            .filter(|&e| e % 2 == 0)
            .fold(Vec::new, |mut v, e| {
                v.push(e);
                v
            })
            .reduce_with(|mut v1, v2| {
                v1.extend(v2);
                v1
            })
            .unwrap();
        println!("by blocks: {:?}", start.elapsed());
        v2
    })
    .expect("failed saving svg");
    let v = svg("rayon_collect.svg", || {
        let start = std::time::Instant::now();
        let v: Vec<i32> = (0..16_000_000)
            .into_par_iter()
            .filter(|&e| e % 2 == 0)
            .collect();
        println!("rayon: {:?}", start.elapsed());
        v
    })
    .expect("failed saving");

    assert_eq!(v, v2);

    println!("******* prefix ********");

    // this code is highly radio-active
    // it is only here for demonstration purposes
    // so brace yourselves
    let mut v = vec![1u64; 100_000_000];
    svg("blocks_prefix.svg", || {
        let start = std::time::Instant::now();
        let init_values = vec![0u64; rayon::current_num_threads()];
        v.par_iter_mut()
            .by_blocks_iter(std::iter::repeat(1_000_000))
            .fold(
                || {
                    let thread = rayon::current_thread_index().unwrap();
                    let init = &init_values[thread];
                    let p = init as *const u64 as *mut u64;
                    let r = unsafe { p.as_mut() }.unwrap();
                    *r = 0;
                    r
                },
                |old_e, e| {
                    *e += *old_e;
                    e
                },
            )
            .reduce_with(|e1, e2| {
                let mut start = e1 as *mut u64;
                let end = e2 as *mut _;
                let size = (end as usize - start as usize) / 8;
                start = unsafe { start.add(1) };
                let slice = unsafe { std::slice::from_raw_parts_mut(start, size as usize) };
                slice.iter_mut().for_each(|e| *e += *e1);
                e2
            });
        println!("by_blocks: {:?}", start.elapsed());
    })
    .unwrap();
    assert!(v.into_iter().eq(1..=100_000_000));

    let mut v = vec![1u64; 100_000_000];
    svg("rayon_prefix.svg", || {
        let start = std::time::Instant::now();
        let init_values = vec![0u64; rayon::current_num_threads()];
        v.par_iter_mut()
            .by_blocks_iter(std::iter::once(std::usize::MAX))
            .fold(
                || {
                    let thread = rayon::current_thread_index().unwrap();
                    let init = &init_values[thread];
                    let p = init as *const u64 as *mut u64;
                    let r = unsafe { p.as_mut() }.unwrap();
                    *r = 0;
                    r
                },
                |old_e, e| {
                    *e += *old_e;
                    e
                },
            )
            .reduce_with(|e1, e2| {
                let mut start = e1 as *mut u64;
                let end = e2 as *mut _;
                let size = (end as usize - start as usize) / 8; // already one less
                start = unsafe { start.add(1) };
                let slice = unsafe { std::slice::from_raw_parts_mut(start, size as usize) };
                slice.iter_mut().for_each(|e| *e += *e1);
                e2
            });
        println!("rayon: {:?}", start.elapsed());
    })
    .unwrap();
    assert!(v.into_iter().eq(1..=100_000_000));
}
