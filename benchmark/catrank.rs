/*
 * Copyright (c) 2013, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

use std;

use common::*;
use catrank_capnp::*;

pub type RequestBuilder<'a> = SearchResultList::Builder<'a>;
pub type ResponseBuilder<'a> = SearchResultList::Builder<'a>;
pub type Expectation = int;
pub type RequestReader<'a> = SearchResultList::Reader<'a>;
pub type ResponseReader<'a> = SearchResultList::Reader<'a>;

pub struct ScoredResult<'a> {
    score : f64,
    result : SearchResult::Reader<'a>
}

static URL_PREFIX : &'static str = "http://example.com";

pub fn setup_request(rng : &mut FastRand, request : SearchResultList::Builder) -> int {
    let count = rng.next_less_than(1000) as uint;
    let mut goodCount : int = 0;

    let list = request.init_results(count);

    for i in range(0, count) {
        let result = list.get(i);
        result.set_score(1000.0 - i as f64);
        let url_size = rng.next_less_than(100) as uint;

        let url_prefix_length = URL_PREFIX.as_bytes().len();
        let url = result.init_url(url_size + url_prefix_length);

        let bytes = url.as_mut_bytes();
        std::io::BufWriter::new(bytes).write(URL_PREFIX.as_bytes()).unwrap();

        for j in range(0, url_size) {
            bytes[j + url_prefix_length] = (97 + rng.next_less_than(26)) as u8;
        }

        let isCat = rng.next_less_than(8) == 0;
        let isDog = rng.next_less_than(8) == 0;
        if isCat && !isDog {
            goodCount += 1;
        }

        let mut snippet = String::from_str(" ");

        let prefix = rng.next_less_than(20) as uint;
        for _ in range(0, prefix) {
            snippet.push_str(WORDS[rng.next_less_than(WORDS.len() as u32) as uint]);
        }
        if isCat { snippet.push_str("cat ") }
        if isDog { snippet.push_str("dog ") }

        let suffix = rng.next_less_than(20) as uint;
        for _ in range(0, suffix) {
            snippet.push_str(WORDS[rng.next_less_than(WORDS.len() as u32) as uint]);
        }

        result.set_snippet(snippet.as_slice());
    }

    goodCount
}

pub fn handle_request(request : SearchResultList::Reader,
                     response : SearchResultList::Builder) {
    let mut scoredResults : Vec<ScoredResult> = Vec::new();

    let results = request.get_results();
    for i in range(0, results.size()) {
        let result = results.get(i);
        let mut score = result.get_score();
        if result.get_snippet().contains(" cat ") {
            score *= 10000.0;
        }
        if result.get_snippet().contains(" dog ") {
            score /= 10000.0;
        }
        scoredResults.push(ScoredResult {score : score, result : result});
    }

    // sort in decreasing order
    scoredResults.sort_by(|v1, v2| { if v1.score < v2.score { std::cmp::Greater } else { std::cmp::Less } });

    let list = response.init_results(scoredResults.len());
    for i in range(0, list.size()) {
        let item = list.get(i);
        let result = scoredResults[i];
        item.set_score(result.score);
        item.set_url(result.result.get_url());
        item.set_snippet(result.result.get_snippet());
    }
}

pub fn check_response(response : SearchResultList::Reader, expectedGoodCount : int) -> bool {
    let mut goodCount : int = 0;
    let results = response.get_results();
    for i in range(0, results.size()) {
        let result = results.get(i);
        if result.get_score() > 1001.0 {
            goodCount += 1;
        } else {
            break;
        }
    }
    return goodCount == expectedGoodCount;
}
