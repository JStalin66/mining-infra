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

Initial ARM64 macOS measurements (CSV in `benchmarks/equihash-20260930-macos.csv`):
valid medians 150.654 us upstream / 71.318 us Zakura (2.11x); invalid medians
74.338 us / 43.314 us (1.72x). These are local microbenchmarks, not observed
fleet latency improvements.

Linux x86_64 AVX2 (Broadwell Xeon E5, 2 vCPU relay host), measured with the
standalone benchmark at low priority without restarting services: valid
medians 360.425 us upstream / 154.257 us Zakura (2.34x); sampled invalid
medians 168.855 us / 91.540 us (1.84x). Raw samples are in
`benchmarks/equihash-20260930-linux-avx2.csv`. This measures verification CPU
cost under host load; it does not measure end-to-end block propagation.
Cloud Build `afa3c501-f1e4-4bff-95a9-983b5c1b1534` built source `6e7c832`,
passed the Linux differential/relay corpus tests, and produced the measured
benchmark. Later commits only adjust documentation, fixture declaration order,
and Docker compiler pins. No production binary was replaced during qualification.

The relay and job-declarator Docker builders require Rust 1.91 because of the
new verifier's minimum Rust version. Both affected packages were checked with
Rust 1.91.0. The translator proxy does not depend on this verifier and retains
its existing compiler pin.

## Other nodes

Zakura 1.5.1 already depends on this exact verifier. Zebra 6.4.2 uses upstream
`equihash 0.3`, with the same 108-byte input and 32-byte nonce call boundary.
ShieldedLabs Zero v29 (7e1b37d), in its `zebra/` subtree, also uses upstream
`equihash 0.3` at that same Rust call boundary. Both are feasible follow-on
candidates. An isolated local comparison against 0.3.0 found no differences
for the eight corpus headers or 11,896 one-byte mutations. On that ARM64 host,
the sampled latest valid header measured 160.942 us / 80.177 us (2.01x),
and its malformed-solution case 79.209 us / 49.015 us (1.62x). This is an
initial library evaluation; full-node builds and qualification remain pending. Keep at least one unchanged full-node implementation as an
independent agreement reference during each candidate experiment. Run
historical/malformed-input differential checks, mainnet acceptance agreement,
and invalid-block rejection before a node rollout. Do not infer node-wide
validation gains from the relay microbenchmark.

Upstream: https://github.com/zakura-core/common/pull/514
