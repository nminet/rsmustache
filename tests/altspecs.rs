extern crate mustache;
use mustache::{
    Template, TemplateMap,
    YamlValue, MapsAndLists
};

use std::{fs, collections::HashMap, cell::RefCell, rc::Rc};
use serde::Deserialize;
use serde_yaml::Mapping as YamlMapping;


#[test]
fn sequence_check_test() -> Result<(), String> {
    run_spec_file("~sequence-check", true)
}

#[test]
fn lambdas_test() -> Result<(), String> {
    run_spec_file("~lambdas", true)
}


fn run_spec_file(name: &str, log: bool) -> Result<(), String> {
    yaml_spec(name)?
        .tests.iter().fold(
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
    partials: Option<YamlMapping>,
    expected: String,
}

fn yaml_spec(name: &str) -> Result<YamlSpecFile, String> {
    let path = format!("tests/altspecs/{}.yml", name);
    let text = fs::read_to_string(path).map_err(
        |err| format!("io: {}", err.to_string())
    )?;
    serde_yaml::from_str::<YamlSpecFile>(&text).map_err(
        |err| format!("yaml: {}", err.to_string())
    )
}

fn run_spec_test(test: &YamlTestSpec, log: bool) -> Result<(), String> {
    let template = Template::from(&test.template)?;
    let partials = if let Some(values) = &test.partials {
        values.iter().map(
            |(name, text)| {
                let name = name.as_str().unwrap();
                let text = text.as_str().unwrap();
                (name, text)
            }
        ).collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };
    let partials = TemplateMap::new(partials)?;
    let data = maps_and_lists(
        &test.data,
        &Rc::from(test.template.as_str())
    );
    let result = template.render_with_partials(
        &data, &partials
    );
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


fn maps_and_lists<'a>(
    yaml: &'a YamlValue,
    template: &Rc<str>
) -> MapsAndLists {
    match yaml {
        YamlValue::Bool(b) => MapsAndLists::bool(*b),
        YamlValue::Number(n) => MapsAndLists::text(&n.to_string()),
        YamlValue::String(s) => MapsAndLists::text(s),
        YamlValue::Mapping(obj) => MapsAndLists::mapping(
            obj.iter().map(|(k, v)|
                (k.as_str().unwrap().to_owned(), maps_and_lists(v, template))
            ).collect::<HashMap<_, _>>()
        ),
        YamlValue::Sequence(seq) => MapsAndLists::sequence(
            seq.iter().map(
                |v| maps_and_lists(v, template)
            ).collect::<Vec<_>>()
        ),
        YamlValue::Tagged(tv) => {
            let tag = tv.tag.to_string();
            let value = tv.value.as_str().unwrap().to_owned();
            match tag.as_str() {
                "!lambda0_str" => MapsAndLists::lambda0(
                    move || value.clone()
                ),
                "!lambda1_str" => MapsAndLists::lambda1(
                    move |s| value.clone().replace("{}", s),
                    template
                ),
                "!lambda0_fn" if value == "counter" => {
                    let counter = RefCell::new(1);
                    MapsAndLists::lambda0(
                        move || {
                            let next = { *counter.borrow() } + 1;
                            counter.replace(next).to_string()
                        }
                    )
                },
                "!lambda1_fn" if value == "check_contents" => {
                    MapsAndLists::lambda1(
                        |s| (if s == "{{x}}" { "yes" } else { "no" }).to_owned(),
                        template
                    )
                },
                _ => MapsAndLists::null()
            }
        },
        _ => MapsAndLists::null()
    }
}
