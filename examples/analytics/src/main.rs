//! Advanced Analytics Example
//!
//! This example demonstrates the comprehensive analytics capabilities of EEYF,
//! including request profiling, predictive analytics, anomaly detection, and
//! usage analytics.

use eeyf::analytics::{Analytics, AnalyticsConfig, RequestProfile};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== EEYF Advanced Analytics Example ===\n");

    // Example 1: Basic Analytics Setup
    println!("1. Creating analytics with custom configuration...");
    let config = AnalyticsConfig::builder()
        .enable_profiling(true)
        .enable_predictions(true)
        .enable_anomaly_detection(true)
        .enable_usage_analytics(true)
        .retention_period(Duration::from_secs(3600)) // 1 hour
        .max_data_points(1000)
        .anomaly_threshold(3.0) // 3 standard deviations
        .prediction_window(Duration::from_secs(300)) // 5 minutes
        .build();
    
    let analytics = Analytics::new(config);
    println!("✓ Analytics engine initialized\n");

    // Example 2: Recording Requests
    println!("2. Recording request data...");
    
    // Simulate normal requests
    for _ in 0..50 {
        analytics.record_request("AAPL", Duration::from_millis(100 + rand::random::<u64>() % 50)).await;
    }
    
    for _ in 0..30 {
        analytics.record_request("GOOGL", Duration::from_millis(120 + rand::random::<u64>() % 60)).await;
    }
    
    for _ in 0..20 {
        analytics.record_request("MSFT", Duration::from_millis(90 + rand::random::<u64>() % 40)).await;
    }
    
    println!("✓ Recorded 100 requests\n");
    
    sleep(Duration::from_millis(100)).await;

    // Example 3: Performance Insights
    println!("3. Getting performance insights...");
    let insights = analytics.get_insights().await;
    
    println!("   Performance Metrics:");
    println!("   - Total requests: {}", insights.total_requests);
    println!("   - Requests per second: {:.2}", insights.requests_per_second);
    println!("   - Average latency: {:?}", insights.average_latency);
    println!("   - P50 latency: {:?}", insights.p50_latency);
    println!("   - P95 latency: {:?}", insights.p95_latency);
    println!("   - P99 latency: {:?}", insights.p99_latency);
    println!("   - Cache hit rate: {:.1}%", insights.cache_hit_rate * 100.0);
    println!("   - Rate limit rate: {:.1}%\n", insights.rate_limit_rate * 100.0);

    // Example 4: Detailed Request Profiling
    println!("4. Recording detailed request profiles...");
    
    // Simulate a request with detailed timing breakdown
    let profile = RequestProfile {
        symbol: "NVDA".to_string(),
        total_duration: Duration::from_millis(250),
        cache_lookup_duration: Some(Duration::from_millis(5)),
        rate_limit_duration: Some(Duration::from_millis(10)),
        network_duration: Some(Duration::from_millis(200)),
        parse_duration: Some(Duration::from_millis(35)),
        cache_hit: false,
        rate_limited: false,
        timestamp: SystemTime::now(),
    };
    
    analytics.record_profile(profile.clone()).await;
    
    println!("   Request Profile for {}:", profile.symbol);
    println!("   - Total duration: {:?}", profile.total_duration);
    println!("   - Cache lookup: {:?}", profile.cache_lookup_duration.unwrap());
    println!("   - Rate limiting: {:?}", profile.rate_limit_duration.unwrap());
    println!("   - Network time: {:?}", profile.network_duration.unwrap());
    println!("   - Parse time: {:?}\n", profile.parse_duration.unwrap());

    // Example 5: Anomaly Detection
    println!("5. Detecting anomalies...");
    
    // Simulate some anomalous requests (very high latency)
    for _ in 0..5 {
        analytics.record_request("TSLA", Duration::from_millis(800 + rand::random::<u64>() % 200)).await;
    }
    
    sleep(Duration::from_millis(100)).await;
    
    if let Some(anomalies) = analytics.detect_anomalies().await {
        println!("   ⚠ {} anomalies detected:", anomalies.len());
        for anomaly in anomalies {
            println!("   - Type: {:?}", anomaly.anomaly_type);
            println!("     Severity: {:.2}", anomaly.severity);
            println!("     Description: {}", anomaly.description);
            if let Some(mitigation) = anomaly.mitigation {
                println!("     Mitigation: {}", mitigation);
            }
            println!();
        }
    } else {
        println!("   ✓ No anomalies detected\n");
    }

    // Example 6: Predictive Analytics
    println!("6. Running predictive analytics...");
    let predictions = analytics.predict_issues().await;
    
    if let Some(exhaustion) = predictions.rate_limit_exhaustion {
        println!("   ⚠ Rate limit may be exhausted in: {:?}", exhaustion);
    } else {
        println!("   ✓ No rate limit exhaustion predicted");
    }
    
    if !predictions.config_suggestions.is_empty() {
        println!("   Configuration suggestions:");
        for suggestion in predictions.config_suggestions {
            println!("   - {}: {} → {}", suggestion.setting, suggestion.current_value, suggestion.suggested_value);
            println!("     Reason: {}", suggestion.reason);
            println!("     Expected impact: {}", suggestion.expected_impact);
        }
    } else {
        println!("   ✓ No configuration changes suggested");
    }
    
    if !predictions.capacity_recommendations.is_empty() {
        println!("   Capacity recommendations:");
        for recommendation in predictions.capacity_recommendations {
            println!("   - {}", recommendation);
        }
    }
    println!();

    // Example 7: Usage Analytics
    println!("7. Analyzing usage patterns...");
    let usage = analytics.get_usage_analytics().await;
    
    println!("   Popular symbols:");
    for (i, (symbol, count)) in usage.popular_symbols.iter().enumerate() {
        println!("   {}. {} ({} requests)", i + 1, symbol, count);
    }
    println!();
    
    if !usage.query_patterns.is_empty() {
        println!("   Query patterns detected:");
        for pattern in usage.query_patterns {
            println!("   - {} (frequency: {})", pattern.description, pattern.frequency);
            if let Some(optimization) = pattern.optimization {
                println!("     Optimization: {}", optimization);
            }
        }
        println!();
    }
    
    if !usage.recommendations.is_empty() {
        println!("   Optimization recommendations:");
        for recommendation in usage.recommendations {
            println!("   - {}", recommendation);
        }
        println!();
    }
    
    println!("   Resource utilization:");
    println!("   - Memory usage: {:.2} MB", usage.resource_utilization.memory_usage_mb);
    println!("   - Cache utilization: {:.1}%", usage.resource_utilization.cache_utilization * 100.0);
    println!("   - Connection pool: {:.1}%", usage.resource_utilization.connection_pool_utilization * 100.0);
    println!("   - API quota: {:.1}%\n", usage.resource_utilization.api_quota_utilization * 100.0);

    // Example 8: Error Tracking
    println!("8. Tracking errors...");
    
    // Simulate some errors
    for _ in 0..5 {
        analytics.record_error().await;
    }
    
    println!("   ✓ Recorded 5 errors");
    
    // Check for error anomalies
    sleep(Duration::from_millis(100)).await;
    if let Some(anomalies) = analytics.detect_anomalies().await {
        let error_anomalies: Vec<_> = anomalies.iter()
            .filter(|a| matches!(a.anomaly_type, eeyf::analytics::AnomalyType::HighErrorRate))
            .collect();
        
        if !error_anomalies.is_empty() {
            println!("   ⚠ High error rate detected!");
        }
    }
    println!();

    // Example 9: Real-Time Monitoring
    println!("9. Simulating real-time monitoring...");
    println!("   Monitoring for 5 seconds with continuous requests...\n");
    
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        // Simulate requests
        let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "NVDA", "AMZN"];
        let symbol = symbols[(rand::random::<u64>() % symbols.len() as u64) as usize];
        let latency = Duration::from_millis(80 + rand::random::<u64>() % 120);
        
        analytics.record_request(symbol, latency).await;
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Get updated insights
    let final_insights = analytics.get_insights().await;
    println!("   Final metrics:");
    println!("   - Total requests: {}", final_insights.total_requests);
    println!("   - Average latency: {:?}", final_insights.average_latency);
    println!("   - Requests per second: {:.2}\n", final_insights.requests_per_second);

    // Example 10: Analytics Summary
    println!("10. Analytics Summary");
    println!("   ═══════════════════════════════════════");
    println!("   Total requests analyzed: {}", final_insights.total_requests);
    println!("   Performance: {:.0}ms avg, {:.0}ms p95", 
             final_insights.average_latency.as_millis(),
             final_insights.p95_latency.as_millis());
    println!("   Request rate: {:.2} req/s", final_insights.requests_per_second);
    
    let final_usage = analytics.get_usage_analytics().await;
    if !final_usage.popular_symbols.is_empty() {
        let top_symbol = &final_usage.popular_symbols[0];
        println!("   Most requested: {} ({} requests)", top_symbol.0, top_symbol.1);
    }
    
    println!("   ═══════════════════════════════════════\n");
    
    println!("✓ Analytics example completed successfully!");
    println!("\nKey Takeaways:");
    println!("• Request profiling provides detailed timing breakdowns");
    println!("• Anomaly detection identifies unusual patterns automatically");
    println!("• Predictive analytics forecasts potential issues");
    println!("• Usage analytics reveals optimization opportunities");
    println!("• Real-time monitoring enables continuous performance tracking");

    Ok(())
}

// Simple random number generation for demonstration
mod rand {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        );
    }
    
    pub fn random<T: From<u64>>() -> T {
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            seed.set(s);
            T::from(s)
        })
    }
}
