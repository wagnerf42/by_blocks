use by_blocks::prelude::*;
use rayon::prelude::*;

fn main() {
    println!("try me with two threads");
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
    let start = std::time::Instant::now();
    assert_eq!(
        (0..100_000_000)
            .into_par_iter()
            .find_first(|x| *x == 49_900_000),
        Some(49_900_000)
    );
    println!("rayon: {:?}", start.elapsed());
}
