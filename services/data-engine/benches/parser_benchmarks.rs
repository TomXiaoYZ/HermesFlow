use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_engine::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use rust_decimal_macros::dec;

fn benchmark_market_data_creation(c: &mut Criterion) {
    c.bench_function("create_standard_market_data", |b| {
        b.iter(|| {
            StandardMarketData::new(
                black_box(DataSourceType::BinanceSpot),
                black_box("BTCUSDT".to_string()),
                black_box(AssetType::Spot),
                black_box(MarketDataType::Trade),
                black_box(dec!(50000.12345678)),
                black_box(dec!(0.001)),
                black_box(1234567890000),
            )
        })
    });
}

fn benchmark_json_serialization(c: &mut Criterion) {
    let data = StandardMarketData::new(
        DataSourceType::BinanceSpot,
        "BTCUSDT".to_string(),
        AssetType::Spot,
        MarketDataType::Trade,
        dec!(50000.12345678),
        dec!(0.001),
        1234567890000,
    );

    c.bench_function("serialize_to_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&data)).unwrap())
    });
}

fn benchmark_json_deserialization(c: &mut Criterion) {
    let json = r#"{
        "source":"BinanceSpot",
        "exchange":"Binance",
        "symbol":"BTCUSDT",
        "asset_type":"Spot",
        "data_type":"Trade",
        "price":"50000.12345678",
        "quantity":"0.001",
        "timestamp":1234567890000,
        "received_at":1234567890100,
        "bid":null,
        "ask":null,
        "high_24h":null,
        "low_24h":null,
        "volume_24h":null,
        "open_interest":null,
        "funding_rate":null,
        "sequence_id":null,
        "raw_data":""
    }"#;

    c.bench_function("deserialize_from_json", |b| {
        b.iter(|| {
            let _data: StandardMarketData = serde_json::from_str(black_box(json)).unwrap();
        })
    });
}

fn benchmark_decimal_operations(c: &mut Criterion) {
    let price1 = dec!(50000.12345678);
    let price2 = dec!(50001.98765432);

    c.bench_function("decimal_addition", |b| {
        b.iter(|| black_box(price1) + black_box(price2))
    });

    c.bench_function("decimal_division", |b| {
        b.iter(|| black_box(price1) / black_box(dec!(2)))
    });
}

criterion_group!(
    benches,
    benchmark_market_data_creation,
    benchmark_json_serialization,
    benchmark_json_deserialization,
    benchmark_decimal_operations
);
criterion_main!(benches);
