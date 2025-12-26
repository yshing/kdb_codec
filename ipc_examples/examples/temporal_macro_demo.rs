//! Example demonstrating the k! macro with temporal (date/time) types.
//!
//! This shows how to create timestamp, date, timespan and other temporal data.

use chrono::prelude::*;
use chrono::Duration;
use kdb_codec::*;

fn main() {
    println!("=== Temporal Types with k! Macro ===\n");

    // ========== Timestamp (DateTime<Utc>) ==========
    println!("--- Timestamps ---");
    
    let timestamp = k!(timestamp: 
        NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_nano_opt(10, 30, 45, 123456789)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap()
    );
    println!("Timestamp: {}", timestamp);
    
    let timestamp_list = k!(timestamp: vec![
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap(),
    ]; @sorted);
    println!("Timestamp list: {}", timestamp_list);

    // ========== Date (NaiveDate) ==========
    println!("\n--- Dates ---");
    
    let date = k!(date: NaiveDate::from_ymd_opt(2024, 12, 25).unwrap());
    println!("Date: {}", date);
    
    let date_list = k!(date: vec![
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    ]);
    println!("Date list: {}", date_list);

    // ========== Month (NaiveDate) ==========
    println!("\n--- Months ---");
    
    let month = k!(month: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    println!("Month: {}", month);
    
    let month_list = k!(month: vec![
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
    ]; @sorted);
    println!("Month list: {}", month_list);

    // ========== Datetime (DateTime<Utc>) ==========
    println!("\n--- Datetime ---");
    
    let datetime = k!(datetime:
        NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_milli_opt(14, 30, 45, 123)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap()
    );
    println!("Datetime: {}", datetime);

    // ========== Timespan (Duration) ==========
    println!("\n--- Timespans ---");
    
    let timespan = k!(timespan: Duration::hours(5) + Duration::minutes(30));
    println!("Timespan: {}", timespan);
    
    let timespan_list = k!(timespan: vec![
        Duration::hours(1),
        Duration::minutes(30),
        Duration::seconds(90),
    ]);
    println!("Timespan list: {}", timespan_list);

    // ========== Time of Day (Duration) ==========
    println!("\n--- Time ---");
    
    let time = k!(time: Duration::hours(14) + Duration::minutes(30) + Duration::seconds(45));
    println!("Time: {}", time);
    
    let time_list = k!(time: vec![
        Duration::hours(9) + Duration::minutes(0),
        Duration::hours(12) + Duration::minutes(30),
        Duration::hours(17) + Duration::minutes(45),
    ]);
    println!("Time list: {}", time_list);

    // ========== Minute (Duration) ==========
    println!("\n--- Minutes ---");
    
    let minute = k!(minute: Duration::minutes(90));
    println!("Minute: {}", minute);
    
    let minute_list = k!(minute: vec![
        Duration::minutes(0),
        Duration::minutes(30),
        Duration::minutes(60),
    ]);
    println!("Minute list: {}", minute_list);

    // ========== Second (Duration) ==========
    println!("\n--- Seconds ---");
    
    let second = k!(second: Duration::seconds(3661));
    println!("Second: {}", second);

    // ========== Trading Data Example ==========
    println!("\n--- Trading Table with Timestamps ---");
    
    let trades = k!(table: {
        "time" => k!(timestamp: vec![
            NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_milli_opt(9, 30, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_milli_opt(9, 30, 1, 500)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15)
                .unwrap()
                .and_hms_milli_opt(9, 30, 3, 250)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
        ]; @sorted),
        "symbol" => k!(sym: vec!["AAPL", "GOOGL", "MSFT"]),
        "price" => k!(float: vec![150.25, 2801.50, 380.75]),
        "size" => k!(long: vec![100, 50, 200])
    });
    
    println!("Trades:\n{}", trades);

    // ========== Time Series Data ==========
    println!("\n--- Time Series Data ---");
    
    let timeseries = k!(dict:
        k!(date: vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        ]; @sorted) =>
        k!(float: vec![100.5, 101.2, 99.8])
    );
    println!("Time series: {}", timeseries);

    println!("\n=== Easy temporal data handling! ===");
}
