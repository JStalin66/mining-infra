//! Compare the optimized verifier and our public wrapper with the old backend.
use zcash_equihash_validator::EquihashValidator;
use zcash_pool_common::fixtures::mainnet_header_and_solution;

fn compare(header: &[u8], solution: &[u8]) {
    let expected =
        equihash::is_valid_solution(200, 9, &header[..108], &header[108..], solution).is_ok();
    assert_eq!(
        equihash_verifier::is_valid_solution(200, 9, &header[..108], &header[108..], solution)
            .is_ok(),
        expected
    );
    assert_eq!(
        EquihashValidator::new()
            .verify_solution(header, solution)
            .is_ok(),
        expected
    );
}

#[test]
fn optimized_backend_agrees_on_real_header_and_single_bit_mutations() {
    let (header, solution) = mainnet_header_and_solution();
    compare(&header, &solution);
    for byte in 0..header.len() {
        for bit in 0..8 {
            let mut changed = header;
            changed[byte] ^= 1 << bit;
            compare(&changed, &solution);
        }
    }
    for byte in 0..solution.len() {
        let mut changed = solution.clone();
        changed[byte] ^= 1 << (byte % 8);
        compare(&header, &changed);
    }
}

#[test]
fn optimized_backend_agrees_on_malformed_solutions() {
    let (header, solution) = mainnet_header_and_solution();
    for len in [0, 1, 1343, 1344, 1345, 2688] {
        compare(&header, &vec![0; len]);
        compare(&header, &vec![255; len]);
    }
    let mut reversed = solution.clone();
    reversed.reverse();
    compare(&header, &reversed);
    let mut duplicated = solution.clone();
    duplicated[672..].copy_from_slice(&solution[..672]);
    compare(&header, &duplicated);
}

#[test]
fn optimized_backend_accepts_historical_mainnet_headers() {
    let corpus: serde_json::Value =
        serde_json::from_str(zcash_pool_common::fixtures::EQUIHASH_MAINNET_HEADERS).unwrap();
    for sample in corpus.as_array().unwrap() {
        let full = hex::decode(sample["header"].as_str().unwrap()).unwrap();
        let mut hash = zcash_pool_common::block_hash::consensus_block_hash(&full);
        hash.reverse();
        assert_eq!(hex::encode(hash), sample["hash"].as_str().unwrap());
        assert!(
            equihash::is_valid_solution(200, 9, &full[..108], &full[108..140], &full[143..])
                .is_ok()
        );
        compare(&full[..140], &full[143..]);
    }
}
