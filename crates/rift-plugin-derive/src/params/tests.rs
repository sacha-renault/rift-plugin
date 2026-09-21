use super::collect::*;

const DEV_INPUT: &str = r#"
        struct OscillatorParam {
            param frequency: FloatParam {
                default: 20f32,
                range: 10f32..20000f32,
            },
            param wave: EnumParam<WaveType> {
                default: WaveType::Square,
            },
            nested_params: {
                param gain: FloatParam {
                    default: 1f32,
                    range: 0f32..2f32,
                },
                param pan: FloatParam {
                    default: 0f32,
                    range: -1f32..1f32,
                },
            }
        }

        struct ChannelParam {
            param gain: FloatParam {
                default: 1f32,
                range: 0f32..2f32,
            },
        }
    "#;

fn parse(input: &str) -> Params {
    syn::parse_str::<Params>(input).expect("should parse")
}

/// Resolve and expand `params`, returning the normalized generated tokens.
fn expanded(params: Params) -> String {
    let resolved = params.resolve().expect("should resolve");
    normalized(&crate::params::expand::expand(resolved).to_string())
}

/// Render any token streamable node for assertions.
fn ts<T: quote::ToTokens>(node: &T) -> String {
    quote::quote!(#node).to_string()
}

#[test]
fn parses_top_level_structs() {
    let params = parse(DEV_INPUT);
    let names: Vec<_> = params.structs.iter().map(|s| s.name.to_string()).collect();
    assert_eq!(names, ["OscillatorParam", "ChannelParam"]);
}

#[test]
fn parses_leaf_params_with_entries() {
    let params = parse(DEV_INPUT);
    let osc = &params.structs[0];

    let FieldValue::Leaf(frequency) = &osc.fields[0].value else {
        panic!("frequency should be a leaf param");
    };
    assert_eq!(ts(&frequency.ty), "FloatParam");
    let keys: Vec<_> = frequency
        .entries
        .iter()
        .map(|e| e.key.to_string())
        .collect();
    assert_eq!(keys, ["default", "range"]);

    let FieldValue::Leaf(wave) = &osc.fields[1].value else {
        panic!("wave should be a leaf param");
    };
    assert_eq!(ts(&wave.ty), "EnumParam < WaveType >");

    let FieldValue::Inline(nested) = &osc.fields[2].value else {
        panic!("nested_params should be inline");
    };
    assert_eq!(nested.len(), 2);
}

#[test]
fn hoists_inline_structs_into_definitions() {
    let resolved = parse(DEV_INPUT).resolve().expect("should resolve");
    let names: Vec<_> = resolved
        .structs
        .iter()
        .map(|s| s.name.to_string())
        .collect();
    assert_eq!(
        names,
        [
            "OscillatorParam",
            "OscillatorParamNestedParams",
            "ChannelParam",
        ]
    );

    // The inline field is now typed by the hoisted struct.
    let osc = resolved
        .structs
        .iter()
        .find(|s| s.name == "OscillatorParam")
        .unwrap();
    let nested = osc
        .fields
        .iter()
        .find(|f| f.name == "nested_params")
        .unwrap();
    assert_eq!(ts(&nested.ty), "OscillatorParamNestedParams");

    // And the hoisted struct kept the original fields.
    let nested = resolved
        .structs
        .iter()
        .find(|s| s.name == "OscillatorParamNestedParams")
        .unwrap();
    let fields: Vec<_> = nested.fields.iter().map(|f| f.name.to_string()).collect();
    assert_eq!(fields, ["gain", "pan"]);
}

#[test]
fn parses_reference_field() {
    let params = parse(
        r#"
            struct Group {
                left: ChannelParam,
            }
        "#,
    );
    let FieldValue::Reference(ty) = &params.structs[0].fields[0].value else {
        panic!("left should be a reference");
    };
    assert_eq!(ts(ty), "ChannelParam");
}

#[test]
fn parses_named_array() {
    let params = parse(
        r#"
            struct Group {
                voices: Array<4, ChannelParam>,
            }
        "#,
    );
    let FieldValue::Array(array) = &params.structs[0].fields[0].value else {
        panic!("voices should be an array");
    };
    assert_eq!(ts(&array.len), "4");

    let ArrayElement::Named(ty) = &array.element else {
        panic!("element should be a named type");
    };
    assert_eq!(ts(ty), "ChannelParam");
}

#[test]
fn parses_const_ident_array_length() {
    let params = parse(
        r#"
            struct Group {
                voices: Array<MAX_VOICES, ChannelParam>,
            }
        "#,
    );
    let FieldValue::Array(array) = &params.structs[0].fields[0].value else {
        panic!("voices should be an array");
    };
    assert_eq!(ts(&array.len), "MAX_VOICES");
}

#[test]
fn hoists_anonymous_array_elements() {
    let resolved = parse(
        r#"
            struct EnvelopeParam {
                param attack: FloatParam { default: 0.1f32 },
                stages: Array<3> {
                    param level: FloatParam { default: 1f32 },
                },
            }
        "#,
    )
    .resolve()
    .expect("should resolve");

    let names: Vec<_> = resolved
        .structs
        .iter()
        .map(|s| s.name.to_string())
        .collect();
    assert_eq!(names, ["EnvelopeParam", "EnvelopeParamStages"]);

    // The array now points at the generated struct and keeps its length.
    let env = &resolved.structs[0];
    let stages = env.fields.iter().find(|f| f.name == "stages").unwrap();
    assert_eq!(normalized(&ts(&stages.ty)), "[EnvelopeParamStages;3]");

    let element = resolved
        .structs
        .iter()
        .find(|s| s.name == "EnvelopeParamStages")
        .unwrap();
    let fields: Vec<_> = element.fields.iter().map(|f| f.name.to_string()).collect();
    assert_eq!(fields, ["level"]);
}

#[test]
fn parses_leaf_without_config() {
    let params = parse("struct Foo { param gain: FloatParam }");
    let FieldValue::Leaf(leaf) = &params.structs[0].fields[0].value else {
        panic!("gain should be a leaf");
    };
    assert_eq!(leaf.kind, LeafKind::Param);
    assert_eq!(ts(&leaf.ty), "FloatParam");
    assert!(leaf.entries.is_empty());
}

#[test]
fn reports_missing_kind_for_leaf() {
    let msg = parse_err("struct Foo { gain: FloatParam { default: 1f32 } }");
    assert!(
        msg.contains("expected a field kind before a parameter definition"),
        "{msg}"
    );
}

#[test]
fn pascal_case_helper() {
    assert_eq!(to_pascal_case("nested_params"), "NestedParams");
    assert_eq!(to_pascal_case("gain"), "Gain");
    assert_eq!(to_pascal_case("a_b_c"), "ABC");
}

fn parse_err(input: &str) -> String {
    syn::parse_str::<Params>(input)
        .expect_err("should fail to parse")
        .to_string()
}

#[test]
fn reports_missing_struct_keyword() {
    let msg = parse_err("Foo { a: FloatParam }");
    assert!(msg.contains("expected a `struct` definition"), "{msg}");
}

#[test]
fn reports_missing_struct_brace() {
    let msg = parse_err("struct Foo;");
    assert!(
        msg.contains("expected `{` to open the struct body"),
        "{msg}"
    );
}

#[test]
fn reports_missing_field_value() {
    let msg = parse_err("struct Foo { gain: }");
    assert!(msg.contains("expected a field value"), "{msg}");
}

#[test]
fn reports_missing_colon() {
    let msg = parse_err("struct Foo { gain FloatParam }");
    assert!(msg.contains("expected `:` after field `gain`"), "{msg}");
}

#[test]
fn reports_missing_comma_between_fields() {
    let msg = parse_err("struct Foo { a: FloatParam b: FloatParam }");
    assert!(msg.contains("expected `,` between fields"), "{msg}");
}

#[test]
fn reports_missing_array_body() {
    let msg = parse_err("struct Foo { stages: Array<3> }");
    assert!(
        msg.contains("expected `{` to open the anonymous `Array` element body"),
        "{msg}"
    );
}

#[test]
fn reports_duplicate_struct() {
    let msg = parse_err("struct Foo {} struct Foo {}");
    assert!(
        msg.contains("struct `Foo` is defined more than once"),
        "{msg}"
    );
}

#[test]
fn reports_duplicate_field() {
    let msg = parse_err("struct Foo { a: FloatParam, a: FloatParam }");
    assert!(
        msg.contains("field `a` is defined more than once in struct `Foo`"),
        "{msg}"
    );
}

#[test]
fn reports_empty_anonymous_group() {
    let msg = parse_err("struct Foo { empty_param: {} }");
    assert!(
        msg.contains("`empty_param` is an empty param group"),
        "{msg}"
    );
}

#[test]
fn reports_empty_array_element_group() {
    let msg = parse_err("struct Foo { stages: Array<3> {} }");
    assert!(msg.contains("`stages` is an empty param group"), "{msg}");
}

/// Collapse all whitespace so token-stream assertions are stable.
fn normalized(tokens: &str) -> String {
    tokens.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn expands_struct_definitions() {
    let params = parse(
        r#"
            struct Child {
                param gain: FloatParam { default: 1f32 },
            }

            struct Parent {
                param freq: FloatParam { default: 20f32, range: 10f32..20000f32 },
                child: Child,
                voices: Array<2, Child>,
            }
        "#,
    );

    let out = expanded(params);

    assert!(out.contains("structChild{gain:FloatParam,}"), "{out}");
    assert!(
        out.contains("structParent{freq:FloatParam,child:Child,voices:[Child;2],}"),
        "{out}"
    );
    // Structs stay path-agnostic: no constructors, ids or modules here.
    assert!(!out.contains("fncreate"), "{out}");
    assert!(!out.contains("ClapId"), "{out}");
}

#[test]
fn expands_hoisted_group_definitions() {
    let params = parse(
        r#"
            struct Envelope {
                stages: Array<3> {
                    param level: FloatParam { default: 1f32 },
                },
            }
        "#,
    );

    let out = expanded(params);

    assert!(
        out.contains("structEnvelope{stages:[EnvelopeStages;3],}"),
        "{out}"
    );
    assert!(
        out.contains("structEnvelopeStages{level:FloatParam,}"),
        "{out}"
    );
}

#[test]
fn expands_generic_field_type() {
    let params =
        parse("struct Foo { param wave: EnumParam<WaveType> { default: WaveType::Square } }");
    let out = expanded(params);

    assert!(
        out.contains("structFoo{wave:EnumParam<WaveType>,}"),
        "{out}"
    );
}

#[test]
fn expands_root_create() {
    let params = parse(
        r#"
            struct ChannelParam {
                param gain: FloatParam { default: 1f32, min: 0f32, max: 1f32 },
            }
            struct OscillatorParam {
                param frequency: FloatParam { default: 20f32 },
                nested: {
                    param pan: FloatParam { default: 0f32 },
                },
            }
            params {
                left: ChannelParam,
                oscillator: Array<2, OscillatorParam>,
            }
        "#,
    );

    let out = expanded(params);

    // Root type definition.
    assert!(
        out.contains("structParameters{left:ChannelParam,oscillator:[OscillatorParam;2],}"),
        "{out}"
    );
    // Ids are derived from module.name, not from a positional cursor.
    assert!(
        out.contains("::rift_plugin::prelude::param_id(\"left\",\"gain\")"),
        "{out}"
    );
    // A fully static path becomes a compile-time `const` in `param_ids`.
    assert!(out.contains("pubconstLEFT_GAIN:"), "{out}");
    assert!(out.contains("pubmodparam_ids"), "{out}");
    // The `module` is already the namespace, so no `_ID` suffix.
    assert!(!out.contains("_ID"), "{out}");
    // A literal `Array` unrolls one module per element.
    assert!(
        out.contains("pubmodoscillator{pubmodi0{pubconstFREQUENCY:"),
        "{out}"
    );
    assert!(
        out.contains("param_id(\"oscillator[0]\",\"frequency\")"),
        "{out}"
    );
    assert!(
        out.contains("param_id(\"oscillator[0].nested\",\"pan\")"),
        "{out}"
    );
    assert!(out.contains("pubmodi1{pubconstFREQUENCY:"), "{out}");
    // The static module path is passed through as a literal.
    assert!(out.contains("Some(\"left\".to_string())"), "{out}");
    // Array element uses a unique index var and an indexed module path.
    assert!(out.contains("::core::array::from_fn(|i0|"), "{out}");
    assert!(
        out.contains("let__module1:String=format!(\"{}[{}]\",\"oscillator\",i0)"),
        "{out}"
    );
    // An array-indexed leaf cannot be a `const`, so it hashes the runtime path.
    assert!(
        out.contains("param_id(__module1.as_str(),\"frequency\")"),
        "{out}"
    );
    // Nested group path is composed from the parent binding.
    assert!(
        out.contains("let__module1:String=format!(\"{}.{}\",__module1,\"nested\")"),
        "{out}"
    );
}

#[test]
fn unrolls_literal_array_ids() {
    let params = parse(
        r#"
            struct Child {
                param gain: FloatParam { default: 1f32 },
            }
            params {
                voices: Array<2, Child>,
            }
        "#,
    );

    let out = expanded(params);

    assert!(
        out.contains("pubmodparam_ids{pubmodvoices{pubmodi0{pubconstGAIN:"),
        "{out}"
    );
    assert!(out.contains("param_id(\"voices[0]\",\"gain\")"), "{out}");
    assert!(out.contains("pubmodi1{pubconstGAIN:"), "{out}");
    assert!(out.contains("param_id(\"voices[1]\",\"gain\")"), "{out}");
}

#[test]
fn indexes_const_length_array_ids_at_runtime() {
    let params = parse(
        r#"
            struct Child {
                param gain: FloatParam { default: 1f32 },
            }
            params {
                voices: Array<MAX_VOICES, Child>,
            }
        "#,
    );

    let out = expanded(params);

    // A non-literal length cannot be unrolled, so the id is a function of the
    // index, using the very module string `create()` builds.
    assert!(
        out.contains("pubmodvoices{pubfngain(index0:usize,)"),
        "{out}"
    );
    assert!(
        out.contains("param_id(&format!(\"voices[{}]\",index0),\"gain\")"),
        "{out}"
    );
}

#[test]
fn reports_unknown_struct_in_root() {
    let err = parse("params { left: Missing }")
        .resolve()
        .expect_err("should fail to resolve")
        .to_string();
    assert!(err.contains("unknown struct `Missing`"), "{err}");
}

#[test]
fn reports_recursive_struct() {
    let err = parse(
        r#"
            struct A { b: B }
            struct B { a: A }
            params { a: A }
        "#,
    )
    .resolve()
    .expect_err("should fail to resolve")
    .to_string();
    assert!(err.contains("`A` contains itself"), "{err}");
}
