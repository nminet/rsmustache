extern crate mustache;
use mustache::{Template, YamlValue};

use std::fs;
use serde::Deserialize;

#[test]
fn xxx_test() -> Result<(), String> {
    run_spec_file("xxx.yml", true)
}

#[test]
fn comments_test() -> Result<(), String> {
    run_spec_file("comments.yml", false)
}

#[test]
fn interpolation_test() -> Result<(), String> {
    run_spec_file("interpolation.yml", false)
}

#[test]
fn sections_test() -> Result<(), String> {
    run_spec_file("sections.yml", false)
}

#[test]
fn inverted_test() -> Result<(), String> {
    run_spec_file("inverted.yml", false)
}

#[test]
fn partials_test() -> Result<(), String> {
    run_spec_file("partials.yml", false)
}

#[test]
fn delimiters_test() -> Result<(), String> {
    run_spec_file("delimiters.yml", false)
}


fn run_spec_file(path: &str, log: bool) -> Result<(), String> {
    yaml_spec(path)?
        .tests
        .iter()
        .fold(
            Ok(()),
            |acc, test| match (acc, run_spec_test(test, log)) {
                (acc, Ok(())) => acc,
                (Ok(()), Err(name)) => Err(format!("specs ({}): {}", path, name)),
                (Err(err), Err(name)) => Err(format!("{}, {}", err, name))
            }
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

fn yaml_spec(name: &str) -> Result<YamlSpecFile, String> {
    let path = format!("tests/specs/{}", name);
    let text = fs::read_to_string(path).map_err(
        |err| format!("io: {}", err.to_string())
    )?;
    serde_yaml::from_str::<YamlSpecFile>(&text).map_err(
        |err| format!("yaml: {}", err.to_string())
    )
}

fn run_spec_test(test: &YamlTestSpec, log: bool) -> Result<(), String> {
    let template = Template::from(&test.template).unwrap();
    let result = template.render(&test.data);
    if result != test.expected {
        if log {
            println!("{}: fail", test.name);
            println!("expected:\n{}", test.expected);
            println!("received:\n{}\n", result);
        };
        Err(test.name.clone())
    } else {
        println!("{}: ok", test.name);
        Ok(())
    }
}
