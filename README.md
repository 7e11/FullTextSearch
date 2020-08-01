## FullTextSearch

7/30/2020 - Starting with this article [Let's build a Full-Text Search engine](https://artem.krylysov.com/blog/2020/07/28/lets-build-a-full-text-search-engine/).
> Full-Text Search (FTS) is a technique for searching text in a collection of documents. A Document can refer to a web page, a newspaper article, an email message, or any structured text.

I used about the first 100MB from this dataset of [abstracts of wikipedia articles](https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-abstract1.xml.gz). 
The full dataset is ~900MB uncompressed.

TODO:
- Persist index to disk
- Search for synonyms for terms
- Store sets of document IDs with [Roaring Bitmaps](https://roaringbitmap.org/)
- Determine result relevance with [tf-idf](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
- Implement [stemming](https://en.wikipedia.org/wiki/Stemming) myself

