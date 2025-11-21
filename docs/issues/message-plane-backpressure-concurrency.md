# Message Plane: Backpressure & Concurrency Controls

- **Status:** TODO
- **Area:** Runtime / Performance
- **Owner:** Triage
- **Context:** Current subscriber walks heads in-process with no rate/concurrency limits; multi-writer publish retries are not tuned.

## Tasks
- Add per-topic reader concurrency limits and rate limiting (configurable).
- Establish publish retry policy (jittered backoff) on CAS conflicts; expose metrics.
- Consider per-connection/page size caps and streaming chunking for large envelopes.
- Benchmarks to ensure new guards don’t regress throughput.

## Definition of Done
- Configurable knobs for reader concurrency and publish retry limits exist, with sensible defaults.
- Benchmarks documented; no observable perf regressions on baseline workloads.
