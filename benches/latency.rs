//! Latency and throughput benchmarks for the key_mapper hot path.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evdev::KeyCode;
use kmrebind::KeyMapper;
use std::collections::HashSet;

fn mapped_keys() -> HashSet<KeyCode> {
    [KeyCode::KEY_DOT, KeyCode::KEY_SLASH].into_iter().collect()
}

/// Throughput: how many process_key_event calls per second.
fn bench_process_key_event_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_mapper");
    group.sample_size(1000);
    group.measurement_time(std::time::Duration::from_secs(3));

    group.bench_function("process_key_event (press)", |b| {
        let mut mapper = KeyMapper::new(mapped_keys());
        b.iter(|| {
            black_box(mapper.process_key_event(KeyCode::KEY_DOT, true));
        });
    });

    group.bench_function("process_key_event (release)", |b| {
        let mut mapper = KeyMapper::new(mapped_keys());
        mapper.process_key_event(KeyCode::KEY_DOT, true);
        b.iter(|| {
            black_box(mapper.process_key_event(KeyCode::KEY_DOT, false));
        });
    });

    // Full cycle: press both, release both (one "click").
    group.bench_function("full_cycle (press dot, press slash, release dot, release slash)", |b| {
        b.iter(|| {
            let mut mapper = KeyMapper::new(mapped_keys());
            mapper.process_key_event(KeyCode::KEY_DOT, true);
            mapper.process_key_event(KeyCode::KEY_SLASH, true);
            mapper.process_key_event(KeyCode::KEY_DOT, false);
            mapper.process_key_event(KeyCode::KEY_SLASH, false);
            black_box(mapper.get_mouse_button_state());
        });
    });

    group.finish();
}

/// Latency: time per single process_key_event (nanoseconds).
fn bench_process_key_event_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_ns");
    group.sample_size(500);
    group.measurement_time(std::time::Duration::from_secs(2));

    group.bench_function("process_key_event", |b| {
        let mut mapper = KeyMapper::new(mapped_keys());
        b.iter(|| {
            black_box(mapper.process_key_event(black_box(KeyCode::KEY_DOT), true));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_process_key_event_throughput,
    bench_process_key_event_latency
);
criterion_main!(benches);
