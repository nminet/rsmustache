extern crate mustache;
use mustache::{Template, YamlValue};

use std::fs;
use serde::Deserialize;

#[test]
fn spec_tests() -> Result<(), String> {
    vec![
        "comments",
        "interpolation",
        "sections",
        "inverted",
        "delimiters"
    ].iter().map(
        |name| run_spec_file(name, false)
    ).fold(
        Result::Ok(()),
        |acc, res| match (acc, res) {
            (acc, Ok(())) => acc,
            (Ok(()), err) => err,
            (Err(err1), Err(err2)) => Err(format!("{}\n{}", err1, err2))
        }
    )
}

#[test]
fn xxx_test() -> Result<(), String> {
    run_spec_file("xxx", true)
}

#[test]
fn comments_test() -> Result<(), String> {
    run_spec_file("comments", true)
}

#[test]
fn interpolation_test() -> Result<(), String> {
    run_spec_file("interpolation", true)
}

#[test]
fn sections_test() -> Result<(), String> {
    run_spec_file("sections", true)
}

#[test]
fn inverted_test() -> Result<(), String> {
    run_spec_file("inverted", true)
}

#[test]
fn partials_test() -> Result<(), String> {
    run_spec_file("partials", true)
}

#[test]
fn delimiters_test() -> Result<(), String> {
    run_spec_file("delimiters", true)
}


fn run_spec_file(name: &str, log: bool) -> Result<(), String> {
    yaml_spec(name)?
        .tests
        .iter()
        .fold(
            Ok(()),
            |acc, test| match (acc, run_spec_test(test, log)) {
                (acc, Ok(())) => acc,
                (Ok(()), Err(err)) => Err(format!("specs ({}): {}", name, err)),
                (Err(err1), Err(err2)) => Err(format!("{}, {}", err1, err2))
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
    let path = format!("tests/specs/{}.yml", name);
    let text = fs::read_to_string(path).map_err(
        |err| format!("io: {}", err.to_string())
    )?;
    serde_yaml::from_str::<YamlSpecFile>(&text).map_err(
        |err| format!("yaml: {}", err.to_string())
    )
}

fn run_spec_test(test: &YamlTestSpec, log: bool) -> Result<(), String> {
    let template = Template::from(&test.template)?;
    let result = template.render(&test.data);
    if result != test.expected {
        if log {
            println!("{}: fail", test.name);
            println!("expected:\n{}", test.expected);
            println!("received:\n{}\n", result);
        };
        Err(test.name.to_owned())
    } else {
        if log {
            println!("{}: ok", test.name);
        }
        Ok(())
    }
}
