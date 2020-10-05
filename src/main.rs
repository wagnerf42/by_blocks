use by_blocks::prelude::*;
use fast_tracer::{svg, FastSubscriber};
use rayon::prelude::*;

fn main() {
    let my_subscriber = FastSubscriber::new();
    tracing::subscriber::set_global_default(my_subscriber).expect("setting tracing default failed");
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
        let v2: Vec<i32> = (0..100_000_000)
            .into_par_iter()
            .by_blocks(std::iter::repeat(8_000_000))
            .filter(|&e| e % 2 == 0)
            .fold(Vec::new, |mut v, e| {
                v.push(e);
                v
            })
            .reduce_iter(Vec::new, |mut v1, mut v2| {
                v1.append(&mut v2);
                v1
            });
        println!("by blocks: {:?}", start.elapsed());
        v2
    })
    .expect("failed saving svg");
    let v = svg("rayon_collect.svg", || {
        let start = std::time::Instant::now();
        let v: Vec<i32> = (0..100_000_000)
            .into_par_iter()
            .filter(|&e| e % 2 == 0)
            .collect();
        println!("rayon: {:?}", start.elapsed());
        v
    })
    .expect("failed saving");

    assert_eq!(v, v2);
}
