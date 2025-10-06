//! Example demonstrating market hours checking
//!
//! This example shows how to:
//! - Check if markets are currently open
//! - Get market status for different exchanges
//! - Find next open/close times
//! - Display trading hours and schedules
//! - Handle holidays and weekends
//! - Use custom holiday calendars
//!
//! Run with: cargo run --example market_hours

use chrono::{Duration, Timelike, Utc};
use eeyf::market_hours::{Exchange, Holiday, MarketHoursChecker, MarketHoursConfig, MarketStatus};

fn main() {
    println!("{}", "=".repeat(80));
    println!("Market Hours Checking Example");
    println!("{}", "=".repeat(80));
    println!();

    // Example 1: Check if markets are currently open
    example1_check_current_status();
    println!();

    // Example 2: Check multiple markets at once
    example2_check_multiple_markets();
    println!();

    // Example 3: Get next open/close times
    example3_next_open_close();
    println!();

    // Example 4: Display trading hours for exchanges
    example4_display_trading_hours();
    println!();

    // Example 5: Check specific time (e.g., last Friday)
    example5_check_specific_time();
    println!();

    // Example 6: Custom holiday calendar
    example6_custom_holidays();
    println!();

    // Example 7: Time until market change
    example7_time_until_change();
    println!();

    // Example 8: Timezone conversions
    example8_timezones();
    println!();

    // Example 9: Lunch breaks for Asian markets
    example9_lunch_breaks();
    println!();
}

fn example1_check_current_status() {
    println!("Example 1: Check Current Market Status");
    println!("{}", "-".repeat(80));

    let checker = MarketHoursChecker::new();

    let exchanges = vec![
        Exchange::NYSE,
        Exchange::NASDAQ,
        Exchange::LSE,
        Exchange::TSE,
    ];

    for exchange in exchanges {
        let status = checker.market_status(exchange);
        let status_str = match &status {
            MarketStatus::Open => "🟢 OPEN",
            MarketStatus::Closed => "🔴 CLOSED",
            MarketStatus::Weekend => "📅 WEEKEND",
            MarketStatus::Holiday(name) => {
                println!("  {} ({}): 🎉 HOLIDAY - {}", exchange.name(), exchange.timezone(), name);
                continue;
            }
            MarketStatus::LunchBreak => "🍱 LUNCH BREAK",
        };

        println!("  {} ({}): {}", exchange.name(), exchange.timezone(), status_str);
    }
}

fn example2_check_multiple_markets() {
    println!("Example 2: Check Multiple Markets at Once");
    println!("{}", "-".repeat(80));

    let checker = MarketHoursChecker::new();
    let exchanges = vec![
        Exchange::NYSE,
        Exchange::NASDAQ,
        Exchange::TSX,
        Exchange::LSE,
        Exchange::EURONEXT,
        Exchange::XETRA,
        Exchange::TSE,
        Exchange::HKEX,
        Exchange::SSE,
        Exchange::ASX,
    ];

    let results = checker.check_markets(&exchanges);

    let mut open_count = 0;
    let mut closed_count = 0;

    for (exchange, status) in results {
        if status.is_open() {
            open_count += 1;
            println!("  🟢 {} is OPEN", exchange.name());
        } else {
            closed_count += 1;
        }
    }

    println!();
    println!("  Summary: {} open, {} closed", open_count, closed_count);
}

fn example3_next_open_close() {
    println!("Example 3: Next Open/Close Times");
    println!("{}", "-".repeat(80));

    let checker = MarketHoursChecker::new();

    let exchanges = vec![Exchange::NYSE, Exchange::LSE, Exchange::TSE];

    for exchange in exchanges {
        let is_open = checker.is_market_open(exchange);

        if is_open {
            if let Some(close_time) = checker.next_close_time(exchange) {
                println!("  {} is currently OPEN", exchange.name());
                println!("    Next close: {}", close_time.format("%Y-%m-%d %H:%M:%S %Z"));
            }
        } else {
            if let Some(open_time) = checker.next_open_time(exchange) {
                println!("  {} is currently CLOSED", exchange.name());
                println!("    Next open:  {}", open_time.format("%Y-%m-%d %H:%M:%S %Z"));
            }
        }
        println!();
    }
}

fn example4_display_trading_hours() {
    println!("Example 4: Display Trading Hours");
    println!("{}", "-".repeat(80));

    let all_exchanges = vec![
        Exchange::NYSE,
        Exchange::NASDAQ,
        Exchange::TSX,
        Exchange::LSE,
        Exchange::EURONEXT,
        Exchange::XETRA,
        Exchange::TSE,
        Exchange::HKEX,
        Exchange::SSE,
        Exchange::ASX,
    ];

    for exchange in all_exchanges {
        let (open, close) = exchange.trading_hours();
        let tz = exchange.timezone();

        print!("  {} ({}): {:02}:{:02} - {:02}:{:02}", 
            exchange.name(), 
            tz, 
            open.hour(), 
            open.minute(), 
            close.hour(), 
            close.minute()
        );

        if let Some((lunch_start, lunch_end)) = exchange.lunch_break() {
            print!(" (Lunch: {:02}:{:02} - {:02}:{:02})", 
                lunch_start.hour(), 
                lunch_start.minute(),
                lunch_end.hour(), 
                lunch_end.minute()
            );
        }

        println!();
    }
}

fn example5_check_specific_time() {
    println!("Example 5: Check Status at Specific Time");
    println!("{}", "-".repeat(80));

    let checker = MarketHoursChecker::new();

    // Check last Friday at 10:00 AM EST
    let last_friday = Utc::now() - Duration::days(7);
    let check_time = last_friday
        .with_timezone(&chrono_tz::America::New_York)
        .with_hour(10)
        .unwrap()
        .with_minute(0)
        .unwrap();

    println!("  Checking NYSE on {}", check_time.format("%Y-%m-%d %H:%M:%S %Z"));

    let status = checker.market_status_at(Exchange::NYSE, &check_time);
    match status {
        MarketStatus::Open => println!("  Status: 🟢 Market was OPEN"),
        MarketStatus::Closed => println!("  Status: 🔴 Market was CLOSED"),
        MarketStatus::Weekend => println!("  Status: 📅 Weekend"),
        MarketStatus::Holiday(name) => println!("  Status: 🎉 Holiday - {}", name),
        MarketStatus::LunchBreak => println!("  Status: 🍱 Lunch Break"),
    }
}

fn example6_custom_holidays() {
    println!("Example 6: Custom Holiday Calendar");
    println!("{}", "-".repeat(80));

    // Create config with custom holidays
    let mut config = MarketHoursConfig::new();

    // Add a custom holiday
    let custom_holiday = Holiday::new(2025, 1, 20, "Martin Luther King Jr. Day (Observed)");
    config.add_holiday(Exchange::NYSE, custom_holiday);

    let checker = MarketHoursChecker::with_config(config);

    // Check the holidays
    let holidays = checker.config().get_holidays(Exchange::NYSE);
    println!("  NYSE Holidays (first 5):");
    for (i, holiday) in holidays.iter().take(5).enumerate() {
        println!("    {}. {}-{:02}-{:02}: {}", 
            i + 1,
            holiday.year, 
            holiday.month, 
            holiday.day, 
            holiday.name
        );
    }

    println!();
    println!("  Total holidays configured: {}", holidays.len());
}

fn example7_time_until_change() {
    println!("Example 7: Time Until Market Change");
    println!("{}", "-".repeat(80));

    let checker = MarketHoursChecker::new();

    let exchanges = vec![Exchange::NYSE, Exchange::LSE, Exchange::TSE];

    for exchange in exchanges {
        if let Some(duration) = checker.time_until_change(exchange) {
            let hours = duration.num_hours();
            let minutes = duration.num_minutes() % 60;
            let seconds = duration.num_seconds() % 60;

            if checker.is_market_open(exchange) {
                println!(
                    "  {} closes in {:02}:{:02}:{:02}",
                    exchange.name(),
                    hours,
                    minutes,
                    seconds
                );
            } else {
                println!(
                    "  {} opens in {:02}:{:02}:{:02}",
                    exchange.name(),
                    hours,
                    minutes,
                    seconds
                );
            }
        }
    }
}

fn example8_timezones() {
    println!("Example 8: Working with Timezones");
    println!("{}", "-".repeat(80));

    let now_utc = Utc::now();

    println!("  Current time in different exchanges:");
    println!("  UTC:       {}", now_utc.format("%H:%M:%S"));

    let exchanges = vec![
        Exchange::NYSE,
        Exchange::LSE,
        Exchange::TSE,
        Exchange::HKEX,
    ];

    for exchange in exchanges {
        let local_time = now_utc.with_timezone(&exchange.timezone());
        println!(
            "  {}: {}",
            exchange.timezone(),
            local_time.format("%H:%M:%S")
        );
    }
}

fn example9_lunch_breaks() {
    println!("Example 9: Lunch Breaks (Asian Markets)");
    println!("{}", "-".repeat(80));

    let asian_markets = vec![Exchange::TSE, Exchange::HKEX, Exchange::SSE];

    for exchange in asian_markets {
        if let Some((lunch_start, lunch_end)) = exchange.lunch_break() {
            println!(
                "  {} has lunch break: {:02}:{:02} - {:02}:{:02}",
                exchange.name(),
                lunch_start.hour(),
                lunch_start.minute(),
                lunch_end.hour(),
                lunch_end.minute()
            );

            // Check if we're currently in lunch break
            let checker = MarketHoursChecker::new();
            let status = checker.market_status(exchange);
            if let MarketStatus::LunchBreak = status {
                println!("    ⚠️ Currently in lunch break!");
            }
        } else {
            println!("  {} does not have a lunch break", exchange.name());
        }
    }
}
