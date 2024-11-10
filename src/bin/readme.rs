// MIT License
//
// Copyright (c) 2023 Dan
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/*!
Example code adapted from the README.md of
[the regex-chunker crate](https://github.com/d2718/regex-chunker).
*/
use regex::bytes::Regex;
use regex_splitter::LendingIterator;
use std::error::Error;

fn example() -> Result<(), Box<dyn Error>> {
    use regex_splitter::RegexSplitter;
    use std::collections::BTreeMap;

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    let mut stdin = std::io::stdin();
    let re = Regex::new(r#"[ "\r\n.,!?:;/]+"#)?;
    let mut chunker = RegexSplitter::new(&mut stdin, &re);

    while let Some(chunk) = chunker.next() {
        let word = String::from_utf8_lossy(&chunk?).to_lowercase();
        *counts.entry(word).or_default() += 1;
    }

    println!("{:#?}", &counts);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    example()
}
