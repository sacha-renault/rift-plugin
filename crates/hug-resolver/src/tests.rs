use super::*;

#[test]
fn basic_variable_replacement() {
    let css = "color: var(--primary);";
    let vars = build_variable_map(&[("primary", "#3498db")]);

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, "color: #3498db;");
}

#[test]
fn multiple_variables() {
    let css = "color: var(--primary); background: var(--bg); border: var(--accent);";
    let vars = build_variable_map(&[
        ("primary", "#3498db"),
        ("bg", "#ffffff"),
        ("accent", "#e74c3c"),
    ]);

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(
        result,
        "color: #3498db; background: #ffffff; border: #e74c3c;"
    );
}

#[test]
fn fallback_values_work() {
    let css = "color: var(--missing, red); background: var(--also-missing, #fff);";
    let vars = HashMap::new(); // Empty variables

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, "color: red; background: #fff;");
}

#[test]
fn variable_overrides_fallback() {
    let css = "color: var(--primary, red);";
    let vars = build_variable_map(&[("primary", "#3498db")]);

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, "color: #3498db;");
}

#[test]
fn missing_variable_keeps_original() {
    let css = "color: var(--nope);";
    let vars = HashMap::new();

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, "color: var(--nope);");
}

#[test]
fn complex_css_with_everything() {
    let css = r#"
        .button {
            background: var(--primary);
            color: var(--text, white);
            border: 1px solid var(--missing);
            padding: var(--spacing, 10px);
        }
    "#;

    let vars = build_variable_map(&[("primary", "#3498db"), ("spacing", "16px")]);

    let CssString(result) = resolve_css_variables(css, &vars);

    assert!(result.contains("background: #3498db;"));
    assert!(result.contains("color: white;"));
    assert!(result.contains("border: 1px solid var(--missing);"));
    assert!(result.contains("padding: 16px;"));
}

#[test]
fn real_world_vizia_example() {
    let css = r#"
        button {
            background-color: var(--bg-secondary);
            color: var(--text-primary);
            border: 1px solid var(--border-default, #ccc);
        }

        button:hover {
            background-color: var(--bg-lighter);
            border-color: var(--accent-primary);
        }
    "#;

    let vars = build_variable_map(&[
        ("bg-secondary", "#343434"),
        ("text-primary", "#f1f1f1"),
        ("bg-lighter", "#404040"),
        ("accent-primary", "#51afef"),
    ]);

    let CssString(result) = resolve_css_variables(css, &vars);

    assert!(result.contains("background-color: #343434;"));
    assert!(result.contains("color: #f1f1f1;"));
    assert!(result.contains("border: 1px solid #ccc;")); // Uses fallback
    assert!(result.contains("background-color: #404040;"));
    assert!(result.contains("border-color: #51afef;"));
}

#[test]
fn edge_cases() {
    let css = "var(--test) var(--another, fallback) normal-text var(--third)";
    let vars = build_variable_map(&[("another", "replaced")]);

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, "var(--test) replaced normal-text var(--third)");
}

#[test]
fn whitespace_in_fallbacks() {
    let css = "color: var(--primary,   red  ); margin: var(--space,  10px 20px  );";
    let vars = HashMap::new();

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, "color: red  ; margin: 10px 20px  ;");
}

#[test]
fn build_variable_map_works() {
    let vars = build_variable_map(&[("primary", "#3498db"), ("secondary", "#e74c3c")]);

    assert_eq!(vars.get("primary"), Some(&"#3498db".to_string()));
    assert_eq!(vars.get("secondary"), Some(&"#e74c3c".to_string()));
    assert_eq!(vars.get("missing"), None);
}

#[test]
fn empty_css_doesnt_break() {
    let CssString(result) = resolve_css_variables("", &HashMap::new());
    assert_eq!(result, "");
}

#[test]
fn no_variables_in_css() {
    let css = "color: red; background: blue;";
    let vars = build_variable_map(&[("unused", "value")]);

    let CssString(result) = resolve_css_variables(css, &vars);
    assert_eq!(result, css);
}
