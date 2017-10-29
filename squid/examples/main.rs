extern crate squid;

use squid::BlockParser;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let file = File::open("examples/demo.sq").unwrap();
    let reader = BufReader::new(&file);
    let parser = BlockParser::new(reader.lines());

    for block in parser {
        println!("{:?}", block);
    }
}
