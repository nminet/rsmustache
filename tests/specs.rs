extern crate mustache;
use mustache::{Template, YamlValue, IntoContext};

use std::fs;
use serde::{Deserialize};


#[test]
fn comments_test() -> Result<(), ()> {
    run_spec_file("tests/specs/comments.yml")
}

#[test]
fn interpolation_test() -> Result<(), ()> {
    run_spec_file("tests/specs/interpolation.yml")
}

#[test]
fn sections_test() -> Result<(), ()> {
    run_spec_file("tests/specs/sections.yml")
}

#[test]
fn inverted_test() -> Result<(), ()> {
    run_spec_file("tests/specs/inverted.yml")
}

#[test]
fn partials_test() -> Result<(), ()> {
    run_spec_file("tests/specs/partials.yml")
}

#[test]
fn delimiters_test() -> Result<(), ()> {
    run_spec_file("tests/specs/delimiters.yml")
}


fn run_spec_file(path: &str) -> Result<(), ()> {
     yaml_spec(path)
        .unwrap()
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
    overview: String,
    tests: Vec<YamlTestSpec>,
}

#[derive(Deserialize, Debug)]
struct YamlTestSpec {
    name: String,
    data: YamlValue,
    template: String,
    expected: String,
}

fn yaml_spec(path: &str) -> Result<YamlSpecFile, String> {
    let text = fs::read_to_string(path).map_err(
        |err| format!("io: {}", err.to_string())
    )?;
    serde_yaml::from_str::<YamlSpecFile>(&text).map_err(
        |err| format!("yaml: {}", err.to_string())
    )
}


fn run_spec_test(test: &YamlTestSpec) -> Result<(), String> {
    let template = Template::from(&test.template).unwrap();
    let context = test.data.into_context();
    let result = template.render(&context);
    if result != test.expected {
        println!("{}: fail", test.name);
        println!("expected:\n{}", test.expected);
        println!("received:\n{}\n", result);
        Err(test.name.clone())
    } else {
        println!("{}: ok", test.name);
        Ok(())
    }
}
