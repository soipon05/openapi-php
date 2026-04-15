use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use openapi_php::cli::GenerateMode;
use openapi_php::config::{Framework, PhpVersion};
use openapi_php::{generator, parser};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser::load_and_resolve");

    for name in ["simple.yaml", "petstore.yaml"] {
        let path = fixture(name);
        group.bench_with_input(BenchmarkId::from_parameter(name), &path, |b, path| {
            b.iter(|| parser::load_and_resolve(path).unwrap());
        });
    }

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let simple = fixture("simple.yaml");
    group.bench_with_input(
        BenchmarkId::new("plain", "simple.yaml"),
        &simple,
        |b, path| {
            b.iter(|| {
                let spec = parser::load_and_resolve(path).unwrap();
                generator::run_dry_filtered(
                    &spec,
                    "App\\Bench",
                    &GenerateMode::All,
                    &Framework::Plain,
                    None,
                    &PhpVersion::Php82,
                    false,
                )
                .unwrap();
            });
        },
    );

    let petstore = fixture("petstore.yaml");
    group.bench_with_input(
        BenchmarkId::new("laravel", "petstore.yaml"),
        &petstore,
        |b, path| {
            b.iter(|| {
                let spec = parser::load_and_resolve(path).unwrap();
                generator::run_dry_filtered(
                    &spec,
                    "App\\Bench",
                    &GenerateMode::All,
                    &Framework::Laravel,
                    None,
                    &PhpVersion::Php82,
                    false,
                )
                .unwrap();
            });
        },
    );

    group.finish();
}

criterion_group!(benches, bench_parse, bench_full_pipeline);
criterion_main!(benches);
