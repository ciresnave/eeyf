use eeyf as yahoo;
use tokio_test;

fn get_history() -> Result<yahoo::YResponse, yahoo::YahooError> {
    let provider = yahoo::YahooConnector::new().unwrap();
    let start = time::OffsetDateTime::UNIX_EPOCH;
    let end = time::OffsetDateTime::now_utc();
    tokio_test::block_on(provider.get_quote_history("EUR=x", start, end))
}

fn main() {
    let quote_history = get_history().unwrap();
    println!("Quote history of USD/EUR FX rate:\n{:#?}", quote_history);
}
