/*
 * This file contains the structure of Crawler and its methods
 */

use hyper_native_tls::NativeTlsClient;
use hyper::net::HttpsConnector;
use hyper::Client;
use hyper::Url;
use select::document::Document;
use std::sync::{Arc, Mutex};
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::Sender;

use function::*;
use error::*;

#[derive(Debug)]
pub struct Crawler {
	client: Client,
	pub bank: Arc<Mutex<HashSet<Url>>>,
	pub queue: Arc<Mutex<VecDeque<Url>>>,
}

impl Crawler {
	pub fn new() -> Crawler {
		let ssl = NativeTlsClient::new().unwrap();
		let connector = HttpsConnector::new(ssl);
		let client = Client::with_connector(connector);
		let bank = HashSet::new();
		let deque: VecDeque<Url> = VecDeque::new();

		let crawler = Crawler {
			client: client,
			bank: Arc::new(Mutex::new(bank)),
			queue: Arc::new(Mutex::new(deque)),
		};
		crawler
	}

	pub fn new_shared(queue: Arc<Mutex<VecDeque<Url>>>,
					  bank: Arc<Mutex<HashSet<Url>>>) -> Crawler {
		let mut crawler = Crawler::new();
		crawler.queue = queue;
		crawler.bank = bank;
		crawler
	}

	#[allow(dead_code)]
	pub fn bank(&self) -> Arc<Mutex<HashSet<Url>>> {
		self.bank.clone()
	}

	#[allow(dead_code)]
	pub fn queue(&self) -> Arc<Mutex<VecDeque<Url>>> {
		self.queue.clone()
	}

	pub fn check_empty(&self) -> bool {
		let queue = lock(&(self.queue)).unwrap();
		queue.is_empty()
	}

	pub fn pop(&self) -> Option<Url> {
		sync_pop_url(&self.queue)
	}

	pub fn push(&self, url: Url) -> Result<()> {
		sync_add_url(&self.queue, &self.bank, url)
	}

	pub fn parse(&self, url: Url, tx: Sender<Vec<Url>>, id: usize) -> Result<(Url)> {
		println!("Crawling {:?}", url);
		let (url, body) = match crawler(url) {
			Ok(t) => t,
			Err(_) => bail!(ErrorKind::CannotParse),			
		};
		let body = String::from_utf8_lossy(body.as_slice()).to_string();
		let doc = Document::from(body.as_str());
		let mut urls = Vec::new();
		let hrefs = scrap_href(&doc, "href");
		for href in hrefs {
			if href.starts_with('#') {
				continue;
			}
			let url = match convert_url(&url, &href) {
				Some(u) => u,
				None => continue,
			};
			println!("To Crawl {}: {:?}", id, url);
			urls.push(url);
		}
		match tx.send(urls.clone()) {
			Ok(_) => {}
			Err(_) => panic!("Cannot send the result to channel!"),
		}
		println!("==========Finish {} thread==========", id);
		Ok(url)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn crawler_set() {
		let crawler = Crawler::new();
		let queue = lock(&crawler.queue).unwrap();
		assert_eq!(queue.len(), 0);
		let bank = lock(&crawler.bank).unwrap();
		assert_eq!(bank.len(), 0);
	}

	#[test]
	fn test_new_shared() {
		let mut bank = HashSet::new();
		let mut deque: VecDeque<Url> = VecDeque::new();

		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		let url3 = Url::parse("https://www.youtube.com").unwrap();
		let url4 = Url::parse("https://www.facebook.com").unwrap();

		bank.insert(url1.clone());
		bank.insert(url2.clone());
		bank.insert(url3.clone());
		bank.insert(url4.clone());
		deque.push_back(url1);
		deque.push_back(url2);
		deque.push_back(url3);
		deque.push_back(url4);

		let b = Arc::new(Mutex::new(bank));
		let d = Arc::new(Mutex::new(deque));

		let crawler = Crawler::new_shared(d, b);
		let dequeue = lock(&crawler.queue).unwrap();
		let hashset = lock(&crawler.bank).unwrap();
		assert_eq!(dequeue.len(), 4);
		assert_eq!(hashset.len(), 4);
	}

	#[test]
	fn test_check_empty() {
		let crawler = Crawler::new();
		assert_eq!(crawler.check_empty(), true);

		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		crawler.push(url1).unwrap();
		crawler.push(url2).unwrap();
		assert_eq!(crawler.check_empty(), false);
	}

	#[test]
	fn test_push() {
		let crawler = Crawler::new();
		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		crawler.push(url1).unwrap();
		crawler.push(url2).unwrap();
		let queue = lock(&crawler.queue).unwrap();
		let bank = lock(&crawler.bank).unwrap();
		assert_eq!(queue.len(), 2);
		assert_eq!(bank.len(), 2);
	}

	#[test]
	fn test_pop() {
		let crawler = Crawler::new();
		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		crawler.push(url1.clone()).unwrap();
		crawler.push(url2.clone()).unwrap();
		assert_eq!(crawler.pop().unwrap(), url1);
		assert_eq!(crawler.pop().unwrap(), url2);
	}
}