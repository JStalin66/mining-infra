//! Interleaved measurements; run `cargo bench -p zcash-equihash-validator --bench equihash_verify`.
use std::{hint::black_box, time::Instant};
use zcash_pool_common::fixtures::mainnet_header_and_solution;

fn main() {
    let (header, solution) = mainnet_header_and_solution();
    let mut invalid = solution.clone();
    invalid[700] ^= 1;
    println!("sample,case,backend,ns_per_verification");
    for sample in 0..12 {
        for (case, solution, valid) in [("valid", &solution, true), ("invalid", &invalid, false)] {
            for backend in if sample % 2 == 0 {
                ["upstream", "zakura"]
            } else {
                ["zakura", "upstream"]
            } {
                let verify = || {
                    let input = black_box(&header[..108]);
                    let nonce = black_box(&header[108..]);
                    let solution = black_box(solution.as_slice());
                    if backend == "upstream" {
                        equihash::is_valid_solution(200, 9, input, nonce, solution).is_ok()
                    } else {
                        equihash_verifier::is_valid_solution(200, 9, input, nonce, solution).is_ok()
                    }
                };
                assert_eq!(verify(), valid);
                let start = Instant::now();
                for _ in 0..2000 {
                    black_box(verify());
                }
                println!(
                    "{sample},{case},{backend},{}",
                    start.elapsed().as_nanos() / 2000
                );
            }
        }
    }
}
