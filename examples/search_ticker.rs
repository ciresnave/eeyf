use tokio_test;
use eeyf as yahoo;

fn search_apple() {
    let provider = yahoo::YahooConnector::new().unwrap();
    let resp = tokio_test::block_on(provider.search_ticker("AAPL")).unwrap();

    println!("All tickers found while searching for 'Apple':");
    for item in resp.quotes {
        println!("{}", item.symbol)
    }
}

fn main() {
    search_apple();
}
