/*
 * This file contains the structure of CrawlerSet and its methods
 */

use hyper::Url;
use std::sync::{Arc, Mutex};
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc;
use std::thread;

use crawler::*;
use function::*;
use error::*;

#[derive(Debug)]
pub struct CrawlerSet {
	set: Vec<Crawler>,
	pub bank: Arc<Mutex<HashSet<Url>>>,
	pub queue: Arc<Mutex<VecDeque<Url>>>,
}

impl CrawlerSet {
	pub fn new() -> CrawlerSet {
		let set = Vec::new();
		let bank = HashSet::new();
		let deque: VecDeque<Url> = VecDeque::new();

		CrawlerSet {
			set: set,
			bank: Arc::new(Mutex::new(bank)),
			queue: Arc::new(Mutex::new(deque)),
		}
	}

	pub fn create_clawers(&mut self, number: usize) {
		self.set.clear();
		for _ in 0..number {
			self.add_crawler();
		}
		if self.set.is_empty() {
			self.add_crawler();
		}
	}

	pub fn add_crawler(&mut self) {
		let bank = self.bank();
		let queue = self.queue();
		self.set.push(Crawler::new_shared(queue, bank));
	}

	pub fn crawl_recursive(&mut self) {
		while !self.is_queue_empty() {
			let mut number = self.queue_length();
			if number > 10 {	//Note: make sure that the number of threads cannot be too many
				number = 10;	//Note: also, we should not use barrier to waste time
			}
			self.create_clawers(number);

			for id in 0..number {
				let (tx, rx) = mpsc::channel();
				let crawler = self.set.pop().unwrap();
				let url = self.pop_url_form_queue();
				thread::spawn(move || crawler.parse(url, tx, id));

				match rx.recv() {
					Ok(urls) => {
						for url in urls {
							let mut bank = lock(&self.bank).unwrap();
							if !bank.contains(&url) {
								bank.insert(url.clone());
								lock(&self.queue).unwrap().push_back(url.clone());
							}
						}
					}
					Err(_) => continue,
				};
				
			}
		}
	}

	pub fn start_crawl(&mut self, url: Url) {
		match self.add_url(url) {
			Ok(_) => self.crawl_recursive(),
			Err(_) => panic!("The argument is not an absolute url!"),
		}
	}

	pub fn pop_url_form_queue(&mut self) -> Url {
		let mut queue = lock(&(self.queue)).unwrap();
		queue.pop_front().unwrap()
	}

	pub fn add_url(&mut self, url: Url) -> Result<()> {
		sync_add_url(&self.queue, &self.bank, url)
	}

	pub fn queue_length(&self) -> usize {
		let queue = lock(&(self.queue)).unwrap();
		queue.len()
	}

	pub fn is_queue_empty(&self) -> bool {
		let queue = lock(&(self.queue)).unwrap();
		queue.is_empty()
	}

	pub fn bank(&self) -> Arc<Mutex<HashSet<Url>>> {
		self.bank.clone()
	}

	pub fn queue(&self) -> Arc<Mutex<VecDeque<Url>>> {
		self.queue.clone()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn crawler_set_test() {
		let crawlerset = CrawlerSet::new();
		let queue = lock(&crawlerset.queue).unwrap();
		assert_eq!(queue.len(), 0);
		let bank = lock(&crawlerset.bank).unwrap();
		assert_eq!(bank.len(), 0);
	}

	#[test]
	fn test_add_url() {
		let mut crawlerset = CrawlerSet::new();
		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		let url3 = Url::parse("https://www.youtube.com").unwrap();
		let url4 = Url::parse("https://www.facebook.com").unwrap();
		crawlerset.add_url(url1).unwrap();
		crawlerset.add_url(url2).unwrap();
		crawlerset.add_url(url3).unwrap();
		crawlerset.add_url(url4).unwrap();

		let queue = lock(&crawlerset.queue).unwrap();
		let bank = lock(&crawlerset.bank).unwrap();
		assert_eq!(queue.len(), 4);
		assert_eq!(bank.len(), 4);
	}

	#[test]
	fn test_pop_url_form_queue() {
		let mut crawlerset = CrawlerSet::new();
		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		let url3 = Url::parse("https://www.youtube.com").unwrap();
		let url4 = Url::parse("https://www.facebook.com").unwrap();
		crawlerset.add_url(url1.clone()).unwrap();
		crawlerset.add_url(url2.clone()).unwrap();
		crawlerset.add_url(url3.clone()).unwrap();
		crawlerset.add_url(url4.clone()).unwrap();

		let url = crawlerset.pop_url_form_queue();
		assert_eq!(url1, url);
		let url = crawlerset.pop_url_form_queue();
		assert_eq!(url2, url);
		let url = crawlerset.pop_url_form_queue();
		assert_eq!(url3, url);
		let url = crawlerset.pop_url_form_queue();
		assert_eq!(url4, url);
	}

	#[test]
	fn test_queue_length() {
		let mut crawlerset = CrawlerSet::new();
		assert_eq!(crawlerset.queue_length(), 0);

		let url1 = Url::parse("https://www.yahoo.com").unwrap();
		let url2 = Url::parse("https://www.google.com").unwrap();
		let url3 = Url::parse("https://www.youtube.com").unwrap();
		let url4 = Url::parse("https://www.facebook.com").unwrap();
		crawlerset.add_url(url1).unwrap();
		crawlerset.add_url(url2).unwrap();
		crawlerset.add_url(url3).unwrap();
		crawlerset.add_url(url4).unwrap();

		assert_eq!(crawlerset.queue_length(), 4);
	}
}