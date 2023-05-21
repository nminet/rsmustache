extern crate mustache;
use mustache::{Template, YamlValue};

use std::fs;
use serde::Deserialize;

#[test]
fn comments_test() -> Result<(), ()> {
    run_spec_file("comments.yml")
}

#[test]
fn interpolation_test() -> Result<(), ()> {
    run_spec_file("interpolation.yml")
}

#[test]
fn sections_test() -> Result<(), ()> {
    run_spec_file("sections.yml")
}

#[test]
fn inverted_test() -> Result<(), ()> {
    run_spec_file("inverted.yml")
}

#[test]
fn delimiters_test() -> Result<(), ()> {
    run_spec_file("delimiters.yml")
}


fn run_spec_file(path: &str) -> Result<(), ()> {
    yaml_spec(path)?
        .tests
        .iter()
        .fold(
            Ok(()),
            |acc, test| match (acc, run_spec_test(test)) {
                (acc, Ok(())) => acc,
                (Ok(()), Err(name)) => Err(format!("  {}", name)),
                (Err(names), Err(name)) => Err(format!("{}\n  {}", names, name))
            }
        ).map_err(
            |names| println!("failed:\n{}", names)
        )
}


#[derive(Deserialize, Debug)]
struct YamlSpecFile {
    tests: Vec<YamlTestSpec>,
}

#[derive(Deserialize, Debug)]
struct YamlTestSpec {
    name: String,
    data: YamlValue,
    template: String,
    expected: String,
}

fn yaml_spec(name: &str) -> Result<YamlSpecFile, ()> {
    let path = format!("tests/specs/{}", name);
    let text = fs::read_to_string(path).map_err(
        |err| println!("io: {}", err.to_string())
    )?;
    serde_yaml::from_str::<YamlSpecFile>(&text).map_err(
        |err| println!("yaml: {}", err.to_string())
    )
}


fn run_spec_test(test: &YamlTestSpec) -> Result<(), String> {
    let template = Template::from(&test.template).unwrap();
    let result = template.render(&test.data);
    if result != test.expected {
        println!("
        {}: fail", test.name);
        println!("expected:\n{}", test.expected);
        println!("received:\n{}\n", result);
        Err(test.name.clone())
    } else {
        println!("{}: ok", test.name);
        Ok(())
    }
}
