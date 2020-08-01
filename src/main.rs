use std::fs::File;
use std::io::{BufReader, stdin};
use serde::Deserialize;
use std::time::Instant;
use regex::{RegexBuilder};
use std::collections::{HashSet, HashMap};
use std::iter::{FromIterator};
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::hash_map::Entry;
use reduce::Reduce;

fn main() {
    let docs = load_documents();
    // let docs_filtered = search_regex(&docs, "cat");
    // this will panic if there are less than 5 results
    // println!("{:?}", &docs_filtered[..5]);

    let now = Instant::now();
    // Mapping from keywords to a list of document ID's
    // (A non-inverted index would be ID's to keywords)
    let mut inverted_index: HashMap<String, HashSet<u32>> = HashMap::new();
    for doc in &docs {
        for token in analyze(doc.text.as_str()) {
            let entry = inverted_index.entry(token);
            match entry {
                Entry::Occupied(mut e) => {
                    e.get_mut().insert(doc.id);
                },
                Entry::Vacant(e) => {
                    let mut set = HashSet::new();
                    set.insert(doc.id);
                    e.insert(set);
                },
            }
        }
    }
    println!("Built index in {} ms", now.elapsed().as_millis());

    // Take user input in a loop and do searches with that text
    println!("Type in some 🔍 keywords");
    let mut input = String::new();
    loop {
        input.clear();
        stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();   //Trim trailing newline
        let now = Instant::now();
        let id_lists = search_index(&inverted_index, input.as_str());
        let intersection: Option<HashSet<u32>> = id_lists.into_iter().reduce(|set1, set2| {
            // copied() maps copy() over the iter.
            set1.intersection(&set2).copied().collect()
        });
        match intersection {
            Some(set) => {
                let docs_text: Vec<String> = set.into_iter()
                    .map(|i| docs.get(i as usize).unwrap())
                    .map(|doc: &Document| format!("📚 {} - {}", doc.title, doc.text)).collect();
                docs_text.iter().for_each(|s| println!("{}", s));
                println!("Index Search for '{}' took {} ms", input, now.elapsed().as_millis());
            },
            None => println!("No Results Found for '{}'", input),
        }
    }
}

/// Takes a query term, breaks it into tokens, and sees which documents have those tokens.
fn search_index(index: &HashMap<String, HashSet<u32>>, text: &str) -> Vec<HashSet<u32>> {
    let mut doc_ids = Vec::new();
    for token in analyze(text) {
        if let Some(set) = index.get(token.as_str()) {
            doc_ids.push(set.clone());
        }
    }
    return doc_ids;
}

/// Combines the filters
fn analyze(text: &str) -> Vec<String> {
    let tokens = tokenize(text);
    let tokens_stopwords = filter_stopwords(tokens);
    let tokens_stemmed = filter_stem(tokens_stopwords);
    return tokens_stemmed;
}

/// Filters out different adjective endings of the same word stems.
/// "fruitlessly" is converted to "fruitless", etc.
/// Note, the resulting stems may not be valid words.
/// "radical" -> "radic"
/// "anarchism" -> "anarch"
fn filter_stem(tokens: Vec<String>) -> Vec<String> {
    let en_stemmer = Stemmer::create(Algorithm::English);
    tokens.into_iter().map(|s| en_stemmer.stem(s.as_str()).into_owned()).collect()
}

/// Filters out common words from a list of tokens
fn filter_stopwords(tokens: Vec<String>) -> Vec<String> {
    // OEC 10 most common words
    // https://en.wikipedia.org/wiki/Most_common_words_in_English
    let stopwords = vec!["a", "and", "be", "have", "i", "in", "of", "that", "the", "to"];
    let stopwords: HashSet<&str> = HashSet::from_iter(stopwords);
    tokens.into_iter().filter(|s| !stopwords.contains(s.as_str())).collect()
}

/// SplitWhitespace is an iterator over non-whitespace substrings of a string.
/// We only want alphanumeric characters in the output. Also remove all empty strings.
/// Will need to be augmented further with stemming & dropping common words.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c| !char::is_ascii_alphanumeric(&c))
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect() }

/// This approach is easy but doesn't scale well. Takes nearly 2 seconds to search through
/// the medium data. The article claims that regex scaling is linear, so with 10x the docs, this search
/// would take 20 seconds.
#[allow(dead_code)]
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
    println!("Regex Search down to {} docs with term '{}' in {} ms.", filtered.len(), term, now.elapsed().as_millis());
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
    println!("Naive Search down to {} docs with term '{}' in {} ms.", filtered.len(), term, now.elapsed().as_millis());
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
    #[serde(skip)]
    id: u32,
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
    let file = File::open("data/enwiki-latest-abstract1.xml").unwrap();
    let file = BufReader::new(file);
    let xml: Feed = serde_xml_rs::from_reader(file).unwrap();
    let mut docs = xml.docs;
    // Add id's to all of the docs
    for (i, doc) in docs.iter_mut().enumerate() {
        doc.id = i as u32;
    }
    println!("Loaded documents {} in {} seconds", docs.len(), now.elapsed().as_secs());
    return docs;
}