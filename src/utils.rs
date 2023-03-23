extern crate num;

use num::{Zero};
use hyper::Uri;

pub fn has_web_protocol(url: &String) -> bool {
    return is_http_url(url) || is_https_url(url);
}

pub fn is_https_url(url: &String) -> bool {
    return url.contains("https://");
}

fn is_http_url(url: &String) -> bool {
    return url.contains("http://");
}

pub fn print_start_info(url: &Uri, requests_per_second: &u32) {
    println!("Target URL: {}\nRequests per second: {}", url, requests_per_second);
}

pub fn set_default_if_negative_or_zero<T: PartialOrd + Zero>(value: &mut T, default_value: T) {
    if *value <= T::zero() {
        *value = default_value;
    }
}