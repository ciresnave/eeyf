use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use eeyf::{YahooError, ErrorContext};
use std::time::Duration;

/// Mock quote data for parsing benchmarks
const MOCK_QUOTE_JSON: &str = r#"{
  "chart": {
    "result": [{
      "meta": {
        "currency": "USD",
        "symbol": "AAPL",
        "exchangeName": "NASDAQ",
        "instrumentType": "EQUITY",
        "firstTradeDate": 345479400,
        "regularMarketTime": 1704484800,
        "regularMarketPrice": 185.64,
        "previousClose": 185.14,
        "dataGranularity": "1d",
        "range": "1d",
        "validRanges": ["1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"]
      },
      "timestamp": [1704484800],
      "indicators": {
        "quote": [{
          "open": [184.35],
          "high": [186.40],
          "low": [183.92],
          "close": [185.64],
          "volume": [48744608]
        }]
      }
    }],
    "error": null
  }
}"#;

const MOCK_LARGE_QUOTE_JSON: &str = r#"{
  "chart": {
    "result": [{
      "meta": {
        "currency": "USD",
        "symbol": "AAPL",
        "exchangeName": "NASDAQ",
        "instrumentType": "EQUITY",
        "firstTradeDate": 345479400,
        "regularMarketTime": 1704484800,
        "regularMarketPrice": 185.64,
        "previousClose": 185.14,
        "dataGranularity": "1d",
        "range": "1y",
        "validRanges": ["1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"]
      },
      "timestamp": [1672531200, 1672617600, 1672704000, 1672790400, 1672876800, 1672963200, 1673049600, 1673136000, 1673222400, 1673308800, 1673395200, 1673481600, 1673568000, 1673654400, 1673740800, 1673827200, 1673913600, 1674000000, 1674086400, 1674172800, 1674259200, 1674345600, 1674432000, 1674518400, 1674604800, 1674691200, 1674777600, 1674864000, 1674950400, 1675036800, 1675123200, 1675209600, 1675296000, 1675382400, 1675468800, 1675555200, 1675641600, 1675728000, 1675814400, 1675900800, 1675987200, 1676073600, 1676160000, 1676246400, 1676332800, 1676419200, 1676505600, 1676592000, 1676678400, 1676764800, 1676851200, 1676937600, 1677024000, 1677110400, 1677196800, 1677283200, 1677369600, 1677456000, 1677542400, 1677628800, 1677715200, 1677801600, 1677888000, 1677974400, 1678060800, 1678147200, 1678233600, 1678320000, 1678406400, 1678492800, 1678579200, 1678665600, 1678752000, 1678838400, 1678924800, 1679011200, 1679097600, 1679184000, 1679270400, 1679356800, 1679443200, 1679529600, 1679616000, 1679702400, 1679788800, 1679875200, 1679961600, 1680048000, 1680134400, 1680220800, 1680307200, 1680393600, 1680480000, 1680566400, 1680652800, 1680739200, 1680825600, 1680912000, 1680998400, 1681084800],
      "indicators": {
        "quote": [{
          "open": [184.35, 183.50, 182.75, 181.90, 183.20, 184.10, 182.80, 183.45, 184.20, 183.75, 182.95, 183.60, 184.30, 183.85, 184.15, 183.40, 182.65, 183.25, 184.05, 183.55, 182.85, 183.70, 184.25, 183.95, 184.50, 183.30, 182.55, 183.15, 183.80, 184.35, 183.65, 182.90, 183.50, 184.10, 183.75, 184.40, 183.20, 182.45, 183.05, 183.70, 184.25, 183.60, 182.80, 183.40, 184.00, 183.50, 184.15, 183.35, 182.60, 183.20, 183.85, 184.30, 183.70, 183.00, 183.55, 184.20, 183.80, 184.45, 183.25, 182.50, 183.10, 183.75, 184.35, 183.65, 182.95, 183.50, 184.15, 183.75, 184.40, 183.20, 182.45, 183.05, 183.70, 184.30, 183.60, 182.85, 183.45, 184.05, 183.55, 184.20, 183.30, 182.55, 183.15, 183.80, 184.40, 183.65, 182.90, 183.50, 184.10, 183.75, 184.35, 183.20, 182.50, 183.10, 183.75, 184.30, 183.65, 183.00, 183.55, 184.20],
          "high": [186.40, 185.60, 184.80, 184.00, 185.30, 186.20, 184.90, 185.55, 186.30, 185.85, 185.05, 185.70, 186.40, 185.95, 186.25, 185.50, 184.75, 185.35, 186.15, 185.65, 184.95, 185.80, 186.35, 186.05, 186.60, 185.40, 184.65, 185.25, 185.90, 186.45, 185.75, 185.00, 185.60, 186.20, 185.85, 186.50, 185.30, 184.55, 185.15, 185.80, 186.35, 185.70, 184.90, 185.50, 186.10, 185.60, 186.25, 185.45, 184.70, 185.30, 185.95, 186.40, 185.80, 185.10, 185.65, 186.30, 185.90, 186.55, 185.35, 184.60, 185.20, 185.85, 186.45, 185.75, 185.05, 185.60, 186.25, 185.85, 186.50, 185.30, 184.55, 185.15, 185.80, 186.40, 185.70, 184.95, 185.55, 186.15, 185.65, 186.30, 185.40, 184.65, 185.25, 185.90, 186.50, 185.75, 185.00, 185.60, 186.20, 185.85, 186.45, 185.30, 184.60, 185.20, 185.85, 186.40, 185.75, 185.10, 185.65, 186.30],
          "low": [183.92, 183.12, 182.32, 181.52, 182.82, 183.72, 182.42, 183.07, 183.82, 183.37, 182.57, 183.22, 183.92, 183.47, 183.77, 183.02, 182.27, 182.87, 183.67, 183.17, 182.47, 183.32, 183.87, 183.57, 184.12, 182.92, 182.17, 182.77, 183.42, 183.97, 183.27, 182.52, 183.12, 183.72, 183.37, 184.02, 182.82, 182.07, 182.67, 183.32, 183.87, 183.22, 182.42, 183.02, 183.62, 183.12, 183.77, 182.97, 182.22, 182.82, 183.47, 183.92, 183.32, 182.62, 183.17, 183.82, 183.42, 184.07, 182.87, 182.12, 182.72, 183.37, 183.97, 183.27, 182.57, 183.12, 183.77, 183.37, 184.02, 182.82, 182.07, 182.67, 183.32, 183.92, 183.22, 182.47, 183.07, 183.67, 183.17, 183.82, 182.92, 182.17, 182.77, 183.42, 184.02, 183.27, 182.52, 183.12, 183.72, 183.37, 183.97, 182.82, 182.12, 182.72, 183.37, 183.92, 183.27, 182.62, 183.17, 183.82],
          "close": [185.64, 184.84, 184.04, 183.24, 184.54, 185.44, 184.14, 184.79, 185.54, 185.09, 184.29, 184.94, 185.64, 185.19, 185.49, 184.74, 183.99, 184.59, 185.39, 184.89, 184.19, 185.04, 185.59, 185.29, 185.84, 184.64, 183.89, 184.49, 185.14, 185.69, 184.99, 184.24, 184.84, 185.44, 185.09, 185.74, 184.54, 183.79, 184.39, 185.04, 185.59, 184.94, 184.14, 184.74, 185.34, 184.84, 185.49, 184.69, 183.94, 184.54, 185.19, 185.64, 185.04, 184.34, 184.89, 185.54, 185.14, 185.79, 184.59, 183.84, 184.44, 185.09, 185.69, 184.99, 184.29, 184.84, 185.49, 185.09, 185.74, 184.54, 183.79, 184.39, 185.04, 185.64, 184.94, 184.19, 184.79, 185.39, 184.89, 185.54, 184.64, 183.89, 184.49, 185.14, 185.74, 184.99, 184.24, 184.84, 185.44, 185.09, 185.69, 184.54, 183.84, 184.44, 185.09, 185.64, 184.99, 184.34, 184.89, 185.54],
          "volume": [48744608, 47523416, 46302224, 45081032, 46791840, 48502648, 47281456, 47992264, 48703072, 48213880, 47502688, 48213496, 48924304, 48434112, 48703920, 47993728, 47282536, 47993344, 48704152, 48213960, 47503768, 48214576, 48925384, 48435192, 49146000, 47993808, 47283616, 47994424, 48705232, 49416040, 48213848, 47503656, 48214464, 48925272, 48435080, 49145888, 47993696, 47283504, 47994312, 48705120, 49415928, 48213736, 47503544, 48214352, 48925160, 48434968, 49145776, 47993584, 47283392, 47994200, 48705008, 49415816, 48213624, 47503432, 48214240, 48925048, 48434856, 49145664, 47993472, 47283280, 47994088, 48704896, 49415704, 48213512, 47503320, 48214128, 48924936, 48434744, 49145552, 47993360, 47283168, 47993976, 48704784, 49415592, 48213400, 47503208, 48214016, 48924824, 48434632, 49145440, 47993248, 47283056, 47993864, 48704672, 49415480, 48213288, 47503096, 48213904, 48924712, 48434520, 49415368, 47993136, 47282944, 47993752, 48704560, 49415256, 48213176, 47502984, 48213792, 48924600]
        }]
      }
    }],
    "error": null
  }
}"#;

/// Benchmark JSON parsing performance
fn bench_quote_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("quote_parsing");
    
    group.bench_function("parse_single_quote", |b| {
        b.iter(|| {
            let _result: Result<serde_json::Value, _> = 
                serde_json::from_str(black_box(MOCK_QUOTE_JSON));
        })
    });
    
    group.bench_function("parse_large_quote", |b| {
        b.iter(|| {
            let _result: Result<serde_json::Value, _> = 
                serde_json::from_str(black_box(MOCK_LARGE_QUOTE_JSON));
        })
    });
    
    // Benchmark parsing with different payload sizes
    for size in [1, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let json = if size == 1 {
                MOCK_QUOTE_JSON
            } else {
                MOCK_LARGE_QUOTE_JSON
            };
            b.iter(|| {
                for _ in 0..size {
                    let _result: Result<serde_json::Value, _> = 
                        serde_json::from_str(black_box(json));
                }
            })
        });
    }
    
    group.finish();
}

/// Benchmark error handling overhead
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    // Benchmark error creation
    group.bench_function("create_error", |b| {
        b.iter(|| {
            let error = YahooError::FetchFailed(black_box("Fetch failed".to_string()));
            black_box(error)
        })
    });
    
    // Benchmark error with context
    group.bench_function("create_error_with_context", |b| {
        b.iter(|| {
            let context = ErrorContext::new()
                .with_symbol(black_box("AAPL"))
                .with_endpoint(black_box("/v8/finance/chart/AAPL"))
                .with_metadata(black_box("test_key"), black_box("test_value"));
            black_box(context)
        })
    });
    
    // Benchmark is_retryable checks
    group.bench_function("check_is_retryable", |b| {
        let errors = vec![
            YahooError::TooManyRequests("Rate limit".to_string()),
            YahooError::ConnectionFailed("Network error".to_string()),
            YahooError::NoResult,
            YahooError::DeserializeFailed("Parse error".to_string()),
        ];
        
        b.iter(|| {
            for error in &errors {
                let _ = black_box(error.is_retryable());
            }
        })
    });
    
    // Benchmark suggested_action generation
    group.bench_function("generate_suggested_action", |b| {
        let error = YahooError::TooManyRequests("Rate limit".to_string());
        b.iter(|| {
            let _ = black_box(error.suggested_action());
        })
    });
    
    group.finish();
}

/// Benchmark string operations
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    
    // Benchmark symbol validation
    group.bench_function("validate_symbol", |b| {
        let symbols = vec!["AAPL", "GOOGL", "MSFT", "INVALID!", "123ABC"];
        b.iter(|| {
            for symbol in &symbols {
                // Simple validation logic
                let is_valid = symbol.chars().all(|c| c.is_alphanumeric() || c == '.');
                black_box(is_valid);
            }
        })
    });
    
    // Benchmark URL construction
    group.bench_function("construct_url", |b| {
        b.iter(|| {
            let symbol = black_box("AAPL");
            let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
            black_box(url)
        })
    });
    
    // Benchmark query parameter building
    group.bench_function("build_query_params", |b| {
        b.iter(|| {
            let params = vec![
                ("interval", "1d"),
                ("range", "1mo"),
                ("includePrePost", "true"),
            ];
            let query: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            black_box(query)
        })
    });
    
    group.finish();
}

/// Benchmark collection operations
fn bench_collection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections");
    
    // Benchmark HashMap lookups
    group.bench_function("hashmap_lookups", |b| {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("AAPL", 185.64);
        map.insert("GOOGL", 142.87);
        map.insert("MSFT", 378.91);
        
        b.iter(|| {
            let _ = black_box(map.get("AAPL"));
            let _ = black_box(map.get("GOOGL"));
            let _ = black_box(map.get("MSFT"));
            let _ = black_box(map.get("INVALID"));
        })
    });
    
    // Benchmark Vec operations
    group.bench_function("vec_operations", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..100 {
                vec.push(i);
            }
            let sum: i32 = vec.iter().sum();
            black_box(sum)
        })
    });
    
    // Benchmark filtering
    group.bench_function("filter_symbols", |b| {
        let symbols = vec![
            "AAPL", "GOOGL", "MSFT", "AMZN", "META",
            "TSLA", "NVDA", "JPM", "V", "WMT",
        ];
        
        b.iter(|| {
            let filtered: Vec<_> = symbols
                .iter()
                .filter(|s| s.len() <= 4)
                .collect();
            black_box(filtered)
        })
    });
    
    group.finish();
}

/// Benchmark timestamp operations
fn bench_timestamp_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamps");
    
    // Benchmark current timestamp
    group.bench_function("get_current_timestamp", |b| {
        use std::time::SystemTime;
        b.iter(|| {
            let now = SystemTime::now();
            black_box(now)
        })
    });
    
    // Benchmark timestamp formatting
    group.bench_function("format_timestamp", |b| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now();
        
        b.iter(|| {
            let duration = now.duration_since(UNIX_EPOCH).unwrap();
            let secs = duration.as_secs();
            black_box(secs)
        })
    });
    
    // Benchmark timestamp comparisons
    group.bench_function("compare_timestamps", |b| {
        use std::time::{SystemTime, Duration};
        let t1 = SystemTime::now();
        let t2 = t1 + Duration::from_secs(60);
        
        b.iter(|| {
            let is_before = black_box(t1 < t2);
            let is_after = black_box(t2 > t1);
            black_box((is_before, is_after))
        })
    });
    
    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");
    
    // Benchmark Arc cloning
    group.bench_function("arc_clone", |b| {
        use std::sync::Arc;
        let data = Arc::new(vec![1, 2, 3, 4, 5]);
        
        b.iter(|| {
            let cloned = data.clone();
            black_box(cloned)
        })
    });
    
    // Benchmark Mutex locking
    group.bench_function("mutex_lock", |b| {
        use std::sync::Mutex;
        let data = Mutex::new(0);
        
        b.iter(|| {
            let mut guard = data.lock().unwrap();
            *guard += 1;
            black_box(*guard)
        })
    });
    
    // Benchmark RwLock read
    group.bench_function("rwlock_read", |b| {
        use std::sync::RwLock;
        let data = RwLock::new(42);
        
        b.iter(|| {
            let guard = data.read().unwrap();
            black_box(*guard)
        })
    });
    
    // Benchmark RwLock write
    group.bench_function("rwlock_write", |b| {
        use std::sync::RwLock;
        let data = RwLock::new(0);
        
        b.iter(|| {
            let mut guard = data.write().unwrap();
            *guard += 1;
            black_box(*guard)
        })
    });
    
    group.finish();
}

/// Benchmark memory allocations
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    
    // Benchmark String allocation
    group.bench_function("allocate_string", |b| {
        b.iter(|| {
            let s = String::from("AAPL");
            black_box(s)
        })
    });
    
    // Benchmark Vec allocation
    group.bench_function("allocate_vec", |b| {
        b.iter(|| {
            let v = vec![1, 2, 3, 4, 5];
            black_box(v)
        })
    });
    
    // Benchmark String concatenation
    group.bench_function("string_concat", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..10 {
                s.push_str(&i.to_string());
            }
            black_box(s)
        })
    });
    
    // Benchmark format! macro
    group.bench_function("format_macro", |b| {
        b.iter(|| {
            let s = format!("Symbol: {}, Price: {}", "AAPL", 185.64);
            black_box(s)
        })
    });
    
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = 
        bench_quote_parsing,
        bench_error_handling,
        bench_string_operations,
        bench_collection_operations,
        bench_timestamp_operations,
        bench_concurrent_operations,
        bench_memory_operations
}

criterion_main!(benches);
