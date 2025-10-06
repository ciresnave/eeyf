use tokio_test;
use eeyf as yahoo;

fn get_history() -> Result<yahoo::YResponse, yahoo::YahooError> {
    let provider = yahoo::YahooConnector::new().unwrap();
    tokio_test::block_on(provider.get_quote_period_interval("AAPL", "1d", "1m", true))
}

fn main() {
    let quote_history = get_history().unwrap();
    //let times = quote_history.chart.result.timestamp;
    //let quotes = quote_history.indicators.quote.q
    println!("Quote history of VTI:\n{:#?}", quote_history);
}
