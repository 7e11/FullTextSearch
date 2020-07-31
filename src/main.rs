use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use std::time::Instant;
use regex::{Regex, RegexBuilder};

fn main() {
    let docs = load_documents();
    let docs_filtered = search_regex(&docs, "cat");
    // this will panic if there are less than 5 results
    println!("{:?}", &docs_filtered[..5]);
}

/// This approach is easy but doesn't scale well. Takes nearly 2 seconds to search through
/// the medium data. The article claims that regex scaling is linear, so with 10x the docs, this search
/// would take 20 seconds.
fn search_regex(docs: &[Document], term: &str) -> Vec<Document> {
    let now = Instant::now();
    let regex = format!(r"\b{}\b", term);
    let re = RegexBuilder::new(regex.as_str())
        .case_insensitive(true)     // Can also use (?i) inside regex
        .build()
        .unwrap();

    let mut filtered: Vec<Document> = Vec::new();
    for doc in docs {
        if re.is_match(doc.text.as_str()) {
            let cloned_doc = doc.clone();
            filtered.push(cloned_doc);
        }
    }
    println!("Regex Filter down to {} docs with term '{}' in {} ms.", filtered.len(), term, now.elapsed().as_millis());
    return filtered;
}


/// Main issues: case-sensitive, matches substrings instead of word boundaries.
/// "cat" matches caterpillar and education, but not Cat.
/// Rust Specific: Takes a String, should really be a &str. Also, make the docs a reference and build
/// a new array of docs entirely.
/// https://stackoverflow.com/questions/40006219/why-is-it-discouraged-to-accept-a-reference-to-a-string-string-vec-vec-o
/// Takes ~300ms for the medium file.
#[allow(dead_code)]
fn search_naive(docs: Vec<Document>, term: String) -> Vec<Document> {
    let now = Instant::now();
    let filtered: Vec<Document> = docs.into_iter().filter(|d| d.text.contains(term.as_str())).collect();
    println!("Naive Filter down to {} docs with term '{}' in {} ms.", filtered.len(), term, now.elapsed().as_millis());
    return filtered;
}

#[derive(Deserialize, Debug)]
struct Feed {
    #[serde(rename = "doc")]
    docs: Vec<Document>
}

#[derive(Deserialize, Debug, Clone)]
struct Document {
    title: String,
    url: String,
    #[serde(rename = "abstract")]
    text: String,
    links: Links,
}

#[derive(Deserialize, Debug, Clone)]
struct Links {
    #[serde(rename = "sublink")]
    links: Vec<Link>,
}

#[derive(Deserialize, Debug, Clone)]
struct Link {
    linktype: String,
    anchor: String,
    link: String,
}

/// Can take up to 750 seconds with just a BufReader.
///
fn load_documents() -> Vec<Document> {
    let now = Instant::now();
    let file = File::open("data/enwiki-latest-abstract1-medium.xml").unwrap();
    let file = BufReader::new(file);
    let xml: Feed = serde_xml_rs::from_reader(file).unwrap();
    println!("Loaded documents {} in {} seconds", xml.docs.len(), now.elapsed().as_secs());
    return xml.docs;
}