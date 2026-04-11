use openapi_php::parser;
use openapi_php::parser::error::{ParseError, ResolveError};
use std::path::Path;

#[test]
fn error_missing_file() {
    let path = Path::new("tests/fixtures/does_not_exist.yaml");
    let err = parser::load(path).unwrap_err();
    let parse_err = err
        .downcast_ref::<ParseError>()
        .expect("expected ParseError");
    assert!(
        matches!(parse_err, ParseError::Io { .. }),
        "expected Io variant, got: {parse_err}"
    );
    assert!(
        err.to_string().contains("does_not_exist.yaml"),
        "error message should include the file path"
    );
}

#[test]
fn error_bad_yaml() {
    let path = Path::new("tests/fixtures/bad_yaml.yaml");
    let err = parser::load(path).unwrap_err();
    let parse_err = err
        .downcast_ref::<ParseError>()
        .expect("expected ParseError");
    assert!(
        matches!(parse_err, ParseError::Yaml { .. }),
        "expected Yaml variant, got: {parse_err}"
    );
    assert!(
        err.to_string().contains("bad_yaml.yaml"),
        "error message should include the file path"
    );
}

#[test]
fn error_unknown_ref() {
    let yaml = r#"
openapi: "3.0.0"
info:
  title: Test
  version: "1.0"
paths: {}
components:
  schemas:
    Foo:
      type: object
      properties:
        bar:
          $ref: '#/components/schemas/NonExistent'
"#;
    let raw: openapi_php::parser::raw::types::RawOpenApi =
        serde_yaml::from_str(yaml).expect("fixture YAML must be valid");
    let err = openapi_php::parser::resolve::resolve(&raw).unwrap_err();
    let resolve_err = err
        .downcast_ref::<ResolveError>()
        .expect("expected ResolveError");
    assert!(
        matches!(resolve_err, ResolveError::UnknownRef { .. }),
        "expected UnknownRef variant, got: {resolve_err}"
    );
    assert!(
        err.to_string().contains("NonExistent"),
        "error message should name the missing schema"
    );
}
