mod response;
mod utils;

use std::process::exit;
use std::sync::{Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed};

use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{Duration, Instant};
use tokio::time;

use hyper::{Client, Uri};
use hyper::client::HttpConnector;

use argparse::{ArgumentParser, Store};

use utils::{is_https_url, has_web_protocol, print_start_info, set_default_if_negative_or_zero};
use response::ResponseTime;

static ERRORS: AtomicUsize = AtomicUsize::new(0);
static RESPONSE: Mutex<ResponseTime> = Mutex::new(ResponseTime::new());

static DEFAULT_RPS: u32 = 1000;
static DEFAULT_MAX_REQUESTS: u32 = 10000;

#[tokio::main]
async fn main() {
    let mut target_url = String::new();

    let mut requests_per_second: u32 = DEFAULT_RPS;
    let mut max_requests: u32 = DEFAULT_MAX_REQUESTS;

    // TODO: move this to a separate function
    {
        let mut argument_parser = ArgumentParser::new();
        argument_parser.set_description("An application for automated stress testing of your APIs");
        argument_parser.refer(&mut target_url)
            .add_option(
                &["-u", "--url"],
                Store,
                "Target URL for benchmark"
            );
        argument_parser.refer(&mut requests_per_second)
            .add_option(
                &["-r", "--rps"],
                Store,
                "Number of requests per second. Default: 1000"
            );
        argument_parser.refer(&mut max_requests)
            .add_option(
                &["-m", "--max"],
                Store,
                "Max number of requests. Default: 10000"
            );

        argument_parser.parse_args_or_exit();
    }

    let url = parse_target_url(target_url);

    set_default_if_negative_or_zero(&mut requests_per_second, DEFAULT_RPS);
    set_default_if_negative_or_zero(&mut max_requests, DEFAULT_MAX_REQUESTS);

    print_start_info(&url, &(requests_per_second as i32));
    run_main_thread(url, requests_per_second as u32, max_requests as u32).await;

    print_result(get_end().await);
}

// FIXME: this thread must stop when all the requests are processed
async fn run_main_thread<'a>(url: Uri, requests_per_second: u32, max_requests: u32) {
    let mut interval = time::interval(Duration::from_micros(1_000_000 / requests_per_second as u64));
    let client = Client::new();

    tokio::spawn(async move {
        loop {
            if RESPONSE.lock().unwrap().get_count() >= max_requests {
                println!("Must leave how");
                break;
            }

            let target_url = url.clone();
            let client = client.clone();

            tokio::spawn(async move {
                get(target_url, client).await;
            });

            interval.tick().await;
        }
    });
}

async fn get(uri: Uri, client: Client<HttpConnector>) {
    let start = Instant::now();

    match client.get(uri).await {
        Ok(res) => {
            if !res.status().is_success() {
                ERRORS.fetch_add(1, Relaxed);
            }
        },
        Err(_) => {
            ERRORS.fetch_add(1, Relaxed);
        }
    }

    RESPONSE.lock().unwrap().add(start.elapsed().as_millis() as u32);
}

async fn get_end() -> Duration {
    let start = Instant::now();
    signal(SignalKind::interrupt()).unwrap().recv().await;
    return start.elapsed();
}

fn parse_target_url(url: String) -> Uri {
    // We don't want the application to DDOS any server if no specific URL was provided
    if url.is_empty() {
        println!("Target URL was not provided, use --help to know the usage of the application!");
        exit(1);
    }

    if is_https_url(&url) {
        println!("The application does not support HTTPS yet!");
        exit(1);
    }

    if !has_web_protocol(&url) {
        return parse_target_url(String::from("http://") + &url);
    }

    let url = url.parse();

    if url.is_err() {
        println!("Couldn't parse the given URL!");
        exit(1)
    }

    return url.unwrap();
}

fn compute_errors_percentage(req: u32, err: &usize) -> f32 {
    let res = (*err as f32 / req as f32) * 100.0;

    if res > 0 as f32 {
        return res;
    }

    return 0 as f32;
}

fn print_result(end: Duration) {
    let response = RESPONSE.lock().unwrap();
    let errors = ERRORS.load(Relaxed);

    print!("\n\n");
    println!("Elapsed:             {:.2?}", end);
    println!("Requests:            {}", response.get_count());
    println!("Errors:              {}", errors);
    println!("Percent of errors:   {:.2}%", compute_errors_percentage(response.get_count(), &errors));
    println!("Response time: \
                \n - Min:              {}ms \
                \n - Max:              {}ms \
                \n - Average:          {}ms", response.get_min(), response.get_max(), response.get_average());
}
