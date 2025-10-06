use eeyf::advanced_cache::*;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};
use tokio;

#[tokio::main]
async fn main() {
    println!("🔍 Testing AdvancedCache methods step by step...");

    let mut config = CacheConfig::default();
    config.performance_config.adaptive_ttl = false;
    config.l1_config.default_ttl = Duration::from_millis(100);

    let cache = AdvancedCache::new(config);
    println!("✅ AdvancedCache created");

    let key = CacheKey {
        symbol: "TSLA".to_string(),
        interval: "5m".to_string(),
        range: "1d".to_string(),
        params: BTreeMap::new(),
    };

    let test_data = b"expired data".to_vec();

    // STEP 1: Test put method with timeout
    println!("🔍 STEP 1: Testing PUT method...");
    match tokio::time::timeout(Duration::from_secs(3), cache.put(key.clone(), test_data)).await {
        Ok(Ok(())) => println!("✅ PUT completed successfully"),
        Ok(Err(e)) => panic!("❌ PUT failed: {:?}", e),
        Err(_) => panic!("🚨 PUT DEADLOCKED after 3 seconds"),
    }

    // STEP 2: Test get method on fresh item
    println!("🔍 STEP 2: Testing GET method (fresh item)...");
    match tokio::time::timeout(Duration::from_secs(3), cache.get(&key)).await {
        Ok(Some(_)) => println!("✅ GET fresh item completed successfully"),
        Ok(None) => panic!("❌ Fresh item not found"),
        Err(_) => panic!("🚨 GET FRESH ITEM DEADLOCKED after 3 seconds"),
    }

    // STEP 3: Wait for expiration
    println!("🔍 STEP 3: Waiting for expiration...");
    tokio::time::sleep(Duration::from_millis(150)).await;

    // STEP 4: Test get method on expired item - THIS IS LIKELY WHERE IT HANGS
    println!("🔍 STEP 4: Testing GET method (expired item)...");
    match tokio::time::timeout(Duration::from_secs(3), cache.get(&key)).await {
        Ok(None) => println!("✅ GET expired item completed - item properly removed"),
        Ok(Some(_)) => panic!("❌ Expired item still exists"),
        Err(_) => panic!("🚨 GET EXPIRED ITEM DEADLOCKED after 3 seconds - FOUND THE BUG!"),
    }

    println!("🎉 All steps completed without deadlock");
}
