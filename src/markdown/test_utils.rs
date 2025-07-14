use std::fs;

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
