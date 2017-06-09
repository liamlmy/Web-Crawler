/*
 * Concurrent Web Crawler in Rust
 * Author: Mingyang Li
 *
 * Code Explanation:
 * In this code, we input an argument of the begining url of the target website
 * We use the concurrent BFS algorithm to find and crawl all urls.
 * First we push the input url into the queue, then use one thread to parse 
 * (pop it out from the dequeue) this url and get all sub-urls from it and push 
 * them into the queue.
 * Then recursively use multiple threads to parse (pop them out) these urls 
 * which are in the queue and push all the parsed sub-urls into the queue 
 * until the queue is empty.
 */

#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate hyper_native_tls;
extern crate select;

mod error;
mod function;
mod crawler;
mod crawler_set;

use function::*;
use crawler_set::*;

fn main() {
    let mut crawlers = CrawlerSet::new();
    let url = readurl();
    crawlers.start_crawl(url);
}
