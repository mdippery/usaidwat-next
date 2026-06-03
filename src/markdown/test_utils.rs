// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Michael Dippery <michael@monkey-robot.com>

use std::fs;

macro_rules! parse_assert_eq {
    ($left:expr , $right:expr) => {
        assert_eq!(parse(&$left), $right);
    };
}

macro_rules! header_tests {
    ($expected:expr) => {
        seq_macro::seq!(N in 1..=6 {
            #[test]
            fn it_removes_header_~N() {
                let header = "#".repeat(N as usize);
                let text = format!("{header} Some Text");
                parse_assert_eq!(text, $expected);
            }
        });
    };
}

pub(crate) use {header_tests, parse_assert_eq};

pub fn load_markdown(file: &str) -> String {
    let file = format!("tests/markdown/{file}.md");
    read_to_string(&file)
}

pub fn load_output(file: &str) -> String {
    let file = format!("tests/markdown/{file}.txt");
    String::from(read_to_string(&file).trim_end())
}

fn read_to_string(file: &str) -> String {
    fs::read_to_string(&file).expect(&format!("could not find test file: {file}"))
}
