# Benchmarks

## Hardware

| | |
|---|---|
| CPU | Apple M3 |
| OS | macOS 14.6 (23.6.0) |
| Rust | stable (release profile, `opt-level = 3`) |

---

## Criterion — library pipeline (in-process, no I/O)

Measured with `cargo bench` (100 samples each).

| Benchmark | Mean | Range |
|---|---|---|
| `parser::load_and_resolve` — `simple.yaml` | **168 µs** | 146 – 196 µs |
| `parser::load_and_resolve` — `petstore.yaml` | **441 µs** | 436 – 448 µs |
| Full pipeline (parse + plain generate) — `simple.yaml` | **298 µs** | 258 – 348 µs |
| Full pipeline (parse + laravel generate) — `petstore.yaml` | **745 µs** | 728 – 767 µs |

The petstore Laravel full-pipeline is the heaviest workload: resolving 10 schemas, 5 endpoints, and rendering 30+ files takes **< 1 ms** in-process.

---

## hyperfine — end-to-end CLI (includes process startup + file I/O)

Measured with `hyperfine --warmup 5 --runs 50`.

| Command | Mean | Min | Max |
|---|---|---|---|
| `simple.yaml` → plain | **19.7 ms** ± 5.3 ms | 14.3 ms | 42.0 ms |
| `petstore.yaml` → plain | **21.8 ms** ± 4.0 ms | 16.0 ms | 35.5 ms |
| `petstore.yaml` → laravel | **24.0 ms** ± 8.5 ms | 18.1 ms | 67.3 ms |

~20 ms end-to-end is dominated by process startup (Rust binary cold-start + dynamic linker), not by the generation logic itself (< 1 ms). Watch mode (`--watch`) amortises this cost by keeping the process alive.

---

## vs. competitors — same spec, three generators

Synthetic spec: **100 endpoints across 20 CRUD resources, 61 schemas**
(`tests/fixtures/large_api.yaml`, ~2,200 lines YAML). Each tool was asked to
produce a PHP client + models from the same input.

### Sample run

Captured 2026-04-18 on the hardware listed at the top of this file
(Apple M3 / macOS 14.6 / arm64 / Docker Engine 28.5.2).
Every number in this table is one snapshot — re-run the script on your own
box to get an apples-to-apples comparison; Docker cold-start in particular
varies widely between platforms.

| Generator | Runtime | Mean | Files produced | Relative |
|---|---|---:|---:|---:|
| **openapi-php 0.1** | Rust (native binary) | **24.3 ms** ± 0.9 ms | 103 | 1.00× |
| jane-php 7.11 | PHP 8.4 (via docker) | 387.1 ms ± 61.9 ms | 238 | 15.93× slower |
| OpenAPI Generator 7.x | Java (via docker) | 1291 ms ± 152 ms | 169 | 53.13× slower |

openapi-php is **~16× faster than jane-php and ~53× faster than OpenAPI Generator**
on this spec. The gap widens on larger specs because the PHP- and JVM-based
competitors pay a fixed cold-start cost before processing begins.

Docker adds roughly 200 ms of startup overhead to the two containerized
benchmarks. Even excluding that, jane-php on a bare metal PHP 8.4 runtime
clocks in around 150–200 ms — still several times slower than openapi-php.

Latest machine-written results (overwritten by every run) land in
`target/bench-results.md`.

---

## How to reproduce

```bash
# Criterion (library benchmarks)
cargo bench

# hyperfine (end-to-end CLI)
cargo build --release
hyperfine --warmup 5 --runs 50 \
  './target/release/openapi-php generate --input tests/fixtures/simple.yaml --output /tmp/bench-simple --framework plain' \
  './target/release/openapi-php generate --input tests/fixtures/petstore.yaml --output /tmp/bench-petstore --framework plain' \
  './target/release/openapi-php generate --input tests/fixtures/petstore.yaml --output /tmp/bench-petstore-laravel --framework laravel'

# Competitor comparison (jane-php + OpenAPI Generator via docker)
./scripts/bench-vs-competitors.sh
```
