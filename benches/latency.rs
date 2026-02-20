//! Latency and throughput benchmarks for the key_mapper hot path.
//!
//! We measure:
//! - **One key, one click**: press + release of a single key (2 events = one "click").
//! - **Two keys, two independent clicks**: key1 press, key1 release, key2 press, key2 release
//!   (4 events = two separate clicks; not chord/rhythm).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evdev::KeyCode;
use kmrebind::KeyMapper;
use std::collections::HashSet;

fn one_key() -> HashSet<KeyCode> {
    [KeyCode::KEY_SPACE].into_iter().collect()
}

fn two_keys() -> HashSet<KeyCode> {
    [KeyCode::KEY_DOT, KeyCode::KEY_SLASH].into_iter().collect()
}

/// One key: one click = press + release (2 events).
fn bench_one_key_one_click(c: &mut Criterion) {
    let mut group = c.benchmark_group("one_key");
    group.sample_size(1000);
    group.measurement_time(std::time::Duration::from_secs(3));

    group.bench_function("one_click (press + release)", |b| {
        b.iter(|| {
            let mut mapper = KeyMapper::new(one_key());
            mapper.process_key_event(KeyCode::KEY_SPACE, true);
            mapper.process_key_event(KeyCode::KEY_SPACE, false);
            black_box(mapper.get_mouse_button_state());
        });
    });

    group.finish();
}

/// Two keys: two independent clicks (key1 press+release, key2 press+release). Not chord.
fn bench_two_keys_two_independent_clicks(c: &mut Criterion) {
    let mut group = c.benchmark_group("two_keys");
    group.sample_size(1000);
    group.measurement_time(std::time::Duration::from_secs(3));

    group.bench_function("two_independent_clicks (key1 press, key1 release, key2 press, key2 release)", |b| {
        b.iter(|| {
            let mut mapper = KeyMapper::new(two_keys());
            mapper.process_key_event(KeyCode::KEY_DOT, true);
            mapper.process_key_event(KeyCode::KEY_DOT, false);
            mapper.process_key_event(KeyCode::KEY_SLASH, true);
            mapper.process_key_event(KeyCode::KEY_SLASH, false);
            black_box(mapper.get_mouse_button_state());
        });
    });

    group.finish();
}

/// Raw throughput: single process_key_event (press or release).
fn bench_process_key_event_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_mapper");
    group.sample_size(1000);
    group.measurement_time(std::time::Duration::from_secs(3));

    group.bench_function("process_key_event (press)", |b| {
        let mut mapper = KeyMapper::new(one_key());
        b.iter(|| {
            black_box(mapper.process_key_event(KeyCode::KEY_SPACE, true));
        });
    });

    group.bench_function("process_key_event (release)", |b| {
        let mut mapper = KeyMapper::new(one_key());
        mapper.process_key_event(KeyCode::KEY_SPACE, true);
        b.iter(|| {
            black_box(mapper.process_key_event(KeyCode::KEY_SPACE, false));
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
        let mut mapper = KeyMapper::new(one_key());
        b.iter(|| {
            black_box(mapper.process_key_event(black_box(KeyCode::KEY_SPACE), true));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_one_key_one_click,
    bench_two_keys_two_independent_clicks,
    bench_process_key_event_throughput,
    bench_process_key_event_latency
);
criterion_main!(benches);
