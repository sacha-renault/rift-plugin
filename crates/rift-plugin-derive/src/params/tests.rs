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

    // `-60` and `6` are coerced to float literals.
    assert!(
        out.contains(".range(const{::rift_plugin::prelude::FloatRange::linear(-(60.0),6.0)})"),
        "{out}"
    );
}

#[test]
fn absent_default_is_left_to_the_builder() {
    let out = expand(
        r#"
        struct P {
            #[param(range = linear(0, 1))]
            pub a: FloatParam,
        }
        "#,
    );

    // No `.default(...)` is emitted, so the builder's kind default applies (which
    // is what lets an enum honor its own `#[default]` variant).
    assert!(!out.contains(".default("), "{out}");
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
fn range_linear_expands_to_a_linear_range() {
    let out = expand(
        r#"
        struct P {
            #[param(range = linear(0, 1))]
            pub a: FloatParam,
        }
        "#,
    );

    assert!(
        out.contains(".range(const{::rift_plugin::prelude::FloatRange::linear(0.0,1.0)})"),
        "{out}"
    );
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

    // Bounds and factor are coerced to floats, and the call is wrapped in a
    // `const` block so the constructor asserts at compile time.
    assert!(
        out.contains(".range(const{::rift_plugin::prelude::FloatRange::skew(20.0,20000.0,3.0)})"),
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

    assert!(
        out.contains(".range(const{::rift_plugin::prelude::FloatRange::exp(20.0,20000.0,3.0)})"),
        "{out}"
    );
}

#[test]
fn allows_const_skew_factor() {
    // A `const` factor is fine; the constructor's assert still runs at compile
    // time because the call is wrapped in a `const` block.
    let out = expand(
        r#"
        struct P {
            #[param(range = skew(0, 1, SKEW))]
            pub a: FloatParam,
        }
        "#,
    );
    assert!(
        out.contains("const{::rift_plugin::prelude::FloatRange::skew(0.0,1.0,(SKEW)asf32)}"),
        "{out}"
    );
}

#[test]
fn int_range_expands_to_a_linear_range() {
    let out = expand(
        r#"
        struct P {
            #[param(range = linear(0, 16))]
            pub steps: IntParam,
        }
        "#,
    );
    // Int bounds are not float-coerced, and there is no curve.
    assert!(
        out.contains(".range(const{::rift_plugin::prelude::IntRange::linear(0,16)})"),
        "{out}"
    );
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
fn unknown_range_constructor_is_forwarded() {
    // The macro forwards the name verbatim; an unknown one becomes a compile
    // error on the generated `FloatRange::log(..)` call.
    let out = expand(
        r#"
        struct P {
            #[param(range = log(0, 1, 2))]
            pub cutoff: FloatParam,
        }
        "#,
    );
    assert!(
        out.contains("const{::rift_plugin::prelude::FloatRange::log("),
        "{out}"
    );
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

#[test]
fn reject_duplicate_id() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            r#"struct P {
                #[param] pub gain: FloatParam,
                #[param(id="gain")] pub gain2: FloatParam,
            }"#,
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Duplicate id: `gain`"), "{err}");

    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            r#"struct P {
                #[param] pub gain: FloatParam,
                #[nested(module="gain")] pub osc: OscillatorParams,
            }"#,
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Duplicate id: `gain`"), "{err}");

    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            r#"struct P {
                #[param(id = "osc")] pub gain: FloatParam,
                #[nested] pub osc: OscillatorParams,
            }"#,
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("Duplicate id: `osc`"), "{err}");
}

#[test]
fn snake_case_default_ids_are_allowed() {
    // The default id is the field name, so `snake_case` has to stay valid.
    super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            r#"struct P {
                #[param] pub wt_position: FloatParam,
                #[nested] pub nested_params: OscillatorParams,
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn reject_non_word_character_in_id() {
    for input in [
        r#"struct P { #[param(id = "a.b")] pub gain: FloatParam }"#,
        r#"struct P { #[param(id = "a b")] pub gain: FloatParam }"#,
        r#"struct P { #[param(id = "a-b")] pub gain: FloatParam }"#,
    ] {
        let err =
            super::parse::Params::from_derive_input(&syn::parse_str::<DeriveInput>(input).unwrap())
                .unwrap_err();
        assert!(err.to_string().contains("is not allowed"), "{err}");
    }
}

#[test]
fn reject_non_word_character_in_module() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(
            r#"struct P { #[nested(module = "a[0]")] pub osc: OscillatorParams }"#,
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("is not allowed"), "{err}");
}

#[test]
fn reject_empty_id() {
    let err = super::parse::Params::from_derive_input(
        &syn::parse_str::<DeriveInput>(r#"struct P { #[param(id = "")] pub gain: FloatParam }"#)
            .unwrap(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("must not be empty"), "{err}");
}
