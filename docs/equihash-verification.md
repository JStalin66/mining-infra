# Zakura Equihash verification

The relay's header PoW validator and the shared `zcash-equihash-validator`
use `zakura-equihash =2.1.0`, the verifier shipped by Zakura 1.5.1. The CPU
miner's optional solver remains on upstream `equihash 0.2`; that version also
serves as the test oracle. Cargo.lock pins the published crate checksum.

The optimized verifier uses postorder traversal, sorted duplicate detection,
and batched BLAKE2b with runtime CPU dispatch. Its fixed 108-byte input and
32-byte nonce match the callers' existing Zcash header layout. No target,
header serialization, forwarding, or acceptance policy is changed.

## Compatibility

Differential tests cover a real header with every header bit flipped, a bit
flip in every solution byte, malformed solution lengths, reordered/duplicated
solutions, and eight historical mainnet headers. The corpus records hashes
and serialized headers fetched from Zebra on 2026-09-30; hashes are recomputed
in tests. Relay tests also cover target rejection and corrupted headers.
The existing relay target-range guard rejects genesis even though both
Equihash verifiers accept its solution; this change preserves that behavior.

The shared validator is also consumed by pool and job-declaration crates.
Their next build inherits the verifier, but this rollout targets relay
validation only. No solver or full-node implementation is changed.

## Reproduction

```sh
cargo test --locked -p zcash-equihash-validator -p sovright-relay -p sovright-p2p-ingress -p sovright-relay-sidecar
cargo bench --locked -p zcash-equihash-validator --bench equihash_verify
```

The benchmark interleaves both backends, reversing order on alternate samples,
with 12 samples of 2,000 verifications each per backend and case. It checks
results before timing and uses `black_box`. The invalid case flips solution
byte 700; it is one rejection workload, not a DoS bound or network benchmark.

Initial macOS measurements (CSV in `benchmarks/equihash-20260930-macos.csv`):
valid medians 150.654 us upstream / 71.318 us Zakura (2.11x); invalid medians
74.338 us / 43.314 us (1.72x). These are local microbenchmarks, not observed
fleet latency improvements. Linux qualification and rollout receipts belong
in the deployment repository.

## Other nodes

Zakura 1.5.1 already depends on this exact verifier. Zebra 6.4.2 uses upstream
`equihash 0.3`, with the same 108-byte input and 32-byte nonce call boundary.
ShieldedLabs Zero v29 (7e1b37d), in its `zebra/` subtree, also uses upstream
`equihash 0.3` at that same Rust call boundary. Both are feasible follow-on
candidates, but need their own 0.3 baseline comparison and qualified builds. Keep at least one unchanged full-node implementation as an
independent agreement reference during each candidate experiment. Run
historical/malformed-input differential checks, mainnet acceptance agreement,
and invalid-block rejection before a node rollout. Do not infer node-wide
validation gains from the relay microbenchmark.

Upstream: https://github.com/zakura-core/common/pull/514
