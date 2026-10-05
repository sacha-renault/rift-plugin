use super::parse::Params;
use syn::DeriveInput;

/// Parse and expand `input`, returning the whitespace-stripped token stream so
/// assertions are insensitive to `quote!` formatting.
fn expand(input: &str) -> String {
    let input: DeriveInput = syn::parse_str(input).expect("input should be valid Rust");
    let params = Params::from_derive_input(&input).expect("input should parse");
    strip(&super::expand::expand(params).to_string())
}

fn strip(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

const NESTED: &str = r#"
    struct MyNestedParams {
        #[param(name = "Pan", range = linear(-1, 1), unit = "pan")]
        pub pan: FloatParam,

        #[nested]
        pub inner: InnerParams,
    }
"#;

const ROOT: &str = r#"
    struct MyParams {
        #[param(name = "Gain", range = linear(-60, 6), unit = "dB")]
        pub gain: FloatParam,

        #[param(default = true, flags = ParamInfoFlags::IS_AUTOMATABLE)]
        pub bypass: BoolParam,

        #[nested(module = "osc")]
        pub osc: MyNestedParams,

        #[nested]
        pub voices: [MyNestedParams; 8],
    }
"#;

#[test]
fn leaf_id_defaults_to_field_ident() {
    let out = expand(ROOT);

    assert!(out.contains("<FloatParam>::builder()"), "{out}");
    // `id` defaults to the *field* ident (not the display name), so renaming a
    // label keeps the id stable. The leaf lives at the root (empty module).
    assert!(
        out.contains(r#"param_id(__module.as_deref().unwrap_or(""),"gain")"#),
        "{out}"
    );
    assert!(
        out.contains(r#".name(::std::string::String::from("Gain"))"#),
        "{out}"
    );
    assert!(out.contains(".maybe_module(__module.clone())"), "{out}");
}

#[test]
fn explicit_id_overrides_name() {
    let out = expand(
        r#"
        struct P {
            #[param(name = "Volume", id = "Gain")]
            pub gain: FloatParam,
        }
        "#,
    );

    assert!(
        out.contains(r#"param_id(__module.as_deref().unwrap_or(""),"Gain")"#),
        "{out}"
    );
    assert!(
        out.contains(r#".name(::std::string::String::from("Volume"))"#),
        "{out}"
    );
}

#[test]
fn int_literals_are_coerced_for_float_params() {
    let out = expand(ROOT);

    assert!(out.contains("min_value(-(60.0)).max_value(6.0)"), "{out}");
    // Absent `default` falls back to `Default::default()`.
    assert!(
        out.contains(".default(::core::default::Default::default())"),
        "{out}"
    );
}

#[test]
fn bool_param_keeps_default_and_flags() {
    let out = expand(ROOT);

    assert!(out.contains(".default(true)"), "{out}");
    assert!(
        out.contains(".flags(ParamInfoFlags::IS_AUTOMATABLE)"),
        "{out}"
    );
}

#[test]
fn nested_uses_module_override_and_field_name() {
    let out = expand(ROOT);

    // `#[nested(module = "osc")]` overrides the field name as the module segment.
    assert!(
        out.contains(r#"::std::format!("{}.{}",__parent,"osc")"#),
        "{out}"
    );
    // `#[nested]` falls back to the field name (`voices`, exercised as the
    // array element module in the generated `format!`).
    assert!(
        out.contains(r#"::std::format!("{}[{}]","voices",__i)"#),
        "{out}"
    );
    assert!(
        out.contains("<MyNestedParams>::create_with_module"),
        "{out}"
    );
}

#[test]
fn nested_array_is_built_with_from_fn_and_indexed_module() {
    let out = expand(ROOT);

    assert!(out.contains("::core::array::from_fn(|__i|"), "{out}");
    assert!(
        out.contains(r#"::std::format!("{}[{}]","voices",__i)"#),
        "{out}"
    );
}

#[test]
fn all_params_walks_leaves_and_nested_fields() {
    let out = expand(ROOT);

    assert!(out.contains("Param::as_ptr(&self.gain)"), "{out}");
    assert!(out.contains("UserParams::all_params(&self.osc)"), "{out}");
    assert!(out.contains("for__iteminself.voices.iter()"), "{out}");
    assert!(out.contains("UserParams::all_params(__item)"), "{out}");
}

#[test]
fn nested_construction_propagates_module_path() {
    let out = expand(NESTED);

    // A leaf inside `MyNestedParams` uses the module passed in by its parent.
    assert!(
        out.contains(r#"param_id(__module.as_deref().unwrap_or(""),"pan")"#),
        "{out}"
    );
    assert!(out.contains("<InnerParams>::create_with_module"), "{out}");
}

#[test]
fn rejects_fields_without_attribute() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>("struct P { pub gain: FloatParam }").unwrap(),
    )
    .unwrap_err();

    assert!(
        err.to_string().contains("`#[param]` or `#[nested]`"),
        "{err}"
    );
}

#[test]
fn rejects_leaf_arrays() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>("struct P { #[param] pub gains: [FloatParam; 2] }").unwrap(),
    )
    .unwrap_err();

    assert!(err.to_string().contains("arrays of leaf params"), "{err}");
}

#[test]
fn rejects_range_on_bool() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            "struct P { #[param(range = linear(0, 1))] pub bypass: BoolParam }",
        )
        .unwrap(),
    )
    .unwrap_err();

    assert!(
        err.to_string().contains("`range` is only supported"),
        "{err}"
    );
}

#[test]
fn range_linear_has_no_scale_call() {
    let out = expand(
        r#"
        struct P {
            #[param(range = linear(0, 1))]
            pub a: FloatParam,
        }
        "#,
    );

    assert!(out.contains("min_value(0.0).max_value(1.0)"), "{out}");
    // Linear is the builder default, so no `.scale(...)` call is emitted.
    assert!(!out.contains(".scale("), "{out}");
}

#[test]
fn range_skew_expands_bounds_and_curve() {
    let out = expand(
        r#"
        struct P {
            #[param(range = skew(20, 20000, 3))]
            pub cutoff: FloatParam,
        }
        "#,
    );

    assert!(out.contains("min_value(20.0).max_value(20000.0)"), "{out}");
    // Integer factor literals are coerced to floats for float params.
    assert!(
        out.contains(".scale(::rift_plugin::prelude::Scale::Skew(3.0))"),
        "{out}"
    );
}

#[test]
fn range_exp_expands_bounds_and_curve() {
    let out = expand(
        r#"
        struct P {
            #[param(range = exp(20, 20000, 3))]
            pub cutoff: FloatParam,
        }
        "#,
    );

    assert!(out.contains("min_value(20.0).max_value(20000.0)"), "{out}");
    assert!(
        out.contains(".scale(::rift_plugin::prelude::Scale::Exponential(3.0))"),
        "{out}"
    );
}

#[test]
fn rejects_non_positive_skew_literal() {
    for source in [
        "struct P { #[param(range = skew(0, 1, 0))] pub a: FloatParam }",
        "struct P { #[param(range = skew(0, 1, 0.0))] pub a: FloatParam }",
        "struct P { #[param(range = skew(0, 1, -2.0))] pub a: FloatParam }",
    ] {
        let err = super::parse::Params::from_derive_input(
            &syn::parse_str::<DeriveInput>(source).unwrap(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("greater than 0"), "{err}");
    }
}

#[test]
fn rejects_bad_exp_literal() {
    for source in [
        "struct P { #[param(range = exp(0, 1, 0))] pub a: FloatParam }",
        "struct P { #[param(range = exp(0, 1, 1.0))] pub a: FloatParam }",
        "struct P { #[param(range = exp(0, 1, -2.0))] pub a: FloatParam }",
    ] {
        let err = super::parse::Params::from_derive_input(
            &syn::parse_str::<DeriveInput>(source).unwrap(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("`exp` factor"), "{err}");
    }
}

#[test]
fn allows_non_literal_skew_factor() {
    // Not a literal, so the macro cannot check it; validation is left to runtime.
    let out = expand(
        r#"
        struct P {
            #[param(range = skew(0, 1, SKEW))]
            pub a: FloatParam,
        }
        "#,
    );
    assert!(out.contains("Scale::Skew("), "{out}");
}

#[test]
fn int_range_is_linear_only() {
    let out = expand(
        r#"
        struct P {
            #[param(range = linear(0, 16))]
            pub steps: IntParam,
        }
        "#,
    );
    // Int bounds are not float-coerced, and no curve is attached.
    assert!(out.contains("min_value(0).max_value(16)"), "{out}");
    assert!(!out.contains(".scale("), "{out}");

    for source in [
        "struct P { #[param(range = skew(0, 16, 2.0))] pub steps: IntParam }",
        "struct P { #[param(range = exp(0, 16, 2.0))] pub steps: IntParam }",
    ] {
        let err = super::parse::Params::from_derive_input(
            &syn::parse_str::<DeriveInput>(source).unwrap(),
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("only supported for float params"),
            "{err}"
        );
    }
}

#[test]
fn rejects_bare_range_syntax() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>("struct P { #[param(range = 0..1)] pub gain: FloatParam }")
            .unwrap(),
    )
    .unwrap_err();

    assert!(err.to_string().contains("`range` expects"), "{err}");
}

#[test]
fn rejects_unknown_range() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            "struct P { #[param(range = log(0, 1, 2))] pub cutoff: FloatParam }",
        )
        .unwrap(),
    )
    .unwrap_err();

    assert!(err.to_string().contains("`range` expects"), "{err}");
}

#[test]
fn rejects_unknown_param_key() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            r#"struct P { #[param(smooth = "exp(5)")] pub gain: FloatParam }"#,
        )
        .unwrap(),
    )
    .unwrap_err();

    assert!(err.to_string().contains("Unknown field: `smooth`"), "{err}");
}
