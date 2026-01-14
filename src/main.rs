#![allow(unused)]

use std::fs;

mod lexer;

fn prepare_line((line_number, mut line): (usize, &str)) -> (usize, Vec<&str>) {
    if let Some(comment) = line.find(';') {
        line = &line[..comment];
    }
    line = line.trim();
    let line = line
        .split([' ', ',', '\r'])
        .filter(|component| !component.is_empty())
        .collect();
    (line_number + 1, line)
}

fn main() {
    let text = fs::read_to_string("program.txt").unwrap();
    let lines = text
        .lines()
        .enumerate()
        .map(prepare_line)
        .filter(|(_, components)| components.len() > 0);
    for line in lines {
        println!("{:?}: {:?}", line.0, lexer::parse_line(line.1))
    }
}
