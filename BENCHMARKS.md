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
```
