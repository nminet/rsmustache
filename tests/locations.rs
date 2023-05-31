extern crate mustache;
use mustache::Template;


#[test]
fn missing_section_at_root() {
    let text = r#"
    {{#section}}some text{{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let location = template.section_location(
        "other"
    );
    assert!(location.is_none());
}

#[test]
fn missing_inner_section() {
    let text = r#"
    {{#section}}{{#sub}}some text{{/sub}}{{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let location = template.section_location(
        "section.other"
    );
    assert!(location.is_none());
}

#[test]
fn inline_section() {
    let text = r#"
    {{#section}}some text{{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let (start, end) = template.section_location(
        "section"
    ).unwrap();
    assert_eq!(&text[start..end], "some text");
}

#[test]
fn inline_sub_section() {
    let text = r#"
    {{#section}}{{#sub}}some text{{/sub}}{{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let (start, end) = template.section_location(
        "section.sub"
    ).unwrap();
    assert_eq!(&text[start..end], "some text");
}

#[test]
fn second_sub_section() {
    let text = r#"
    {{#section}}{{#sub1}}text1{{/sub1}}{{#sub2}}text2{{/sub2}}{{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let (start, end) = template.section_location(
        "section.sub2"
    ).unwrap();
    assert_eq!(&text[start..end], "text2");
}

#[test]
fn standalone_tag() {
    let text = r#"
    {{#section}}  
text
    {{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let (start, end) = template.section_location(
        "section"
    ).unwrap();
    assert_eq!(&text[start..end], "text\n");
}

#[test]
fn standalone_tag_with_inner_section() {
    let text = r#"
    {{#section}}  {{#sub}}  
text
    {{/sub}}  {{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let (start, end) = template.section_location(
        "section"
    ).unwrap();
    assert_eq!(&text[start..end], "{{#sub}}  \ntext\n    {{/sub}}");
}

#[test]
fn section_with_dotted_name() {
    let text = r#"
    {{#section}}{{#sub.x}}{{#y}}some text{{/y}}{{/sub.x}}{{/section}}
    "#;
    let template = Template::from(text).unwrap();
    let (start, end) = template.section_location(
        "section.sub.x.y"
    ).unwrap();
    assert_eq!(&text[start..end], "some text");
}
