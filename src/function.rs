/*
 * This file contains the basic functions
 */

use hyper_native_tls::NativeTlsClient;
use hyper::net::HttpsConnector;
use hyper::Client;
use hyper::Url;
use select::document::Document;
use select::predicate::Attr;
use std::env;
use std::io::Read;
use hyper::client::IntoUrl;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use error::*;

/* lock the argument */
pub fn lock<T>(mutex: &Arc<Mutex<T>>) -> Result<MutexGuard<T>> {
	match mutex.lock() {
		Ok(t) => Ok(t),
		Err(e) => bail!(ErrorKind::PoisonError(e.to_string())),
	}
}

/* lock the queue and then pop a element from it */
pub fn sync_pop_url(queue: &Arc<Mutex<VecDeque<Url>>>) -> Option<Url> {
	let mut queue = lock(queue).unwrap();
	queue.pop_front()
}

/* lock the queue and bank, then push a element into the queue and insert the same element into the bank */
pub fn sync_add_url(queue: &Arc<Mutex<VecDeque<Url>>>, bank: &Arc<Mutex<HashSet<Url>>>, url: Url) -> Result<()> {
	let mut queue = lock(queue)?;
	let mut bank = lock(bank)?;

	if !bank.contains(&url) {
		bank.insert(url.clone());
		queue.push_back(url.clone());
	}

	Ok(())
}

/* find all "href" attribute in the document */
pub fn scrap_href(doc: &Document, attr: &str) -> Vec<String> {
	let mut attrs = Vec::new();
	let nodes = doc.find(Attr(attr, ()));		//find<P: Predicate>(&self, predicate: P)Returns a Selection 
												//containing nodes passing the givrn predicate p
	for node in nodes.iter() {
		let attr = match node.attr("href") {
			Some(a) => a.to_string(),
			None => continue,
		};
		attrs.push(attr);
	}
	attrs
}

/* read the argument as an url */
pub fn readurl() -> Url {       		//read the input argument as an absolute url
	match env::args().nth(1) {
    	Some(url_str) => {
    		match Url::parse(&url_str) {
    			Ok(url) => {   				
    				url
    			}
    			Err(_) => panic!("Panic!: The argument is not an url!"),
    		}
    	}
    	None => panic!("Panic!: It lacks an input argument!"),
    }
}

/* convert the href attribute to an proper url */
pub fn convert_url(url: &Url, href: &str) -> Option<Url> {
    let url = if href.starts_with("//") {
        let scheme = url.scheme();              //Return the scheme of this URL, lower-cased, as an ASCII string 
                                                //without the ':' delimiter and following string.
        match format!("{}:{}", scheme, href).into_url() {       //Consumes the object, trying to return a Url.
            Ok(u) => u,
            _ => return None,
        }
    } else if href.starts_with("https") {
        match href.into_url() {
            Ok(u) => return Some(u),
            _ => return None,
        }
    } else if href.starts_with('/') {
        let mut url = url.clone();
        url.set_path(href);                     //Change this URL's path
        url
    } else if href.starts_with("javascript") {
        return None;
    } else {
        let path = url.path();
        if path.ends_with(href) {
            return None;
        }
        let mut url = url.clone();
        let href = format_url(format!("{}/{}", url, href));
        url.set_path(&href);
        url
    };
    Some(url)
}

/* change the argument as an proper url */
pub fn format_url<S: AsRef<str>>(url: S) -> String {
	let mut result = String::new();
	let url = url.as_ref();
	let mut last_char = ' ';
	for ch in url.chars() {
		if ch == '/' && last_char == '/' {
			continue;
		}
		result.push(ch);
		last_char = ch;
	}
	result
}

/* read the url and get the body */
pub fn crawler(url: Url) -> Result<(Url, Vec<u8>)> {
	let ssl = match NativeTlsClient::new() {
		Ok(t) => t,
		Err(_) => panic!("Panic!: NativeTlsClient get some problems!"),
	};
	let connector = HttpsConnector::new(ssl);
	let client = Client::with_connector(connector);
	let mut response = client.get(url.clone()).send()?;
	let mut buf = Vec::new();
	let body = match response.read_to_end(&mut buf) {
		Ok(_) => buf,
		Err(e) => bail!(e),
	};
	Ok((url, body))
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_format_url() {
		let url = "https://www.google.com";
		let result = format_url(url);
		assert_eq!("https:/www.google.com", result);

		let url = "www.google.com/xxx/6666";
		let result = format_url(url);
		assert_eq!("www.google.com/xxx/6666", result);

		let url = "/www.google.com////6666/xyz/";
		let result = format_url(url);
		assert_eq!("/www.google.com/6666/xyz/", result);
	}

	#[test]
	fn test_convert_url() {
		let url = Url::parse("https://www.google.com").unwrap();
		let href = "https://www.youtube.com";
		let result = convert_url(&url, href).unwrap();
		let test = Url::parse("https://www.youtube.com").unwrap();
		assert_eq!(test, result);

		let url = Url::parse("https://www.google.com").unwrap();
		let href = "/map/Evanston";
		let result = convert_url(&url, href).unwrap();
		let test = Url::parse("https://www.google.com/map/Evanston").unwrap();
		assert_eq!(test, result);

		let url = Url::parse("https://www.google.com").unwrap();
		let href = "javascript/map/Evanston";
		let result = convert_url(&url, href);
		assert_eq!(result, None);
	}
}