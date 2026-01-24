use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_engine::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use rust_decimal_macros::dec;

fn create_test_data() -> StandardMarketData {
    StandardMarketData::new(
        DataSourceType::BinanceSpot,
        "BTCUSDT".to_string(),
        AssetType::Spot,
        MarketDataType::Trade,
        dec!(50000.12345678),
        dec!(0.001),
        chrono::Utc::now().timestamp_millis(),
    )
}

fn benchmark_redis_key_generation(c: &mut Criterion) {
    let data = create_test_data();

    c.bench_function("generate_redis_key", |b| {
        b.iter(|| {
            format!(
                "market:{}:{}:latest",
                black_box(&data.source),
                black_box(&data.symbol)
            )
        })
    });
}

fn benchmark_data_cloning(c: &mut Criterion) {
    let data = create_test_data();

    c.bench_function("clone_market_data", |b| b.iter(|| black_box(&data).clone()));
}

fn benchmark_batch_preparation(c: &mut Criterion) {
    let mut batch = Vec::with_capacity(1000);
    for _ in 0..1000 {
        batch.push(create_test_data());
    }

    c.bench_function("prepare_1000_row_batch", |b| {
        b.iter(|| {
            let _prepared = batch.to_vec();
        })
    });
}

criterion_group!(
    benches,
    benchmark_redis_key_generation,
    benchmark_data_cloning,
    benchmark_batch_preparation
);
criterion_main!(benches);
