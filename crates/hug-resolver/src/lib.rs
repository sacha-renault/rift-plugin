#[cfg(test)]
mod tests;

use regex::Regex;
use std::collections::HashMap;
use vizia::util::IntoCssStr;

pub fn resolve_css_variables_from_list(css_string: &str, vars: &[(&str, &str)]) -> CssString {
    resolve_css_variables(css_string, &build_variable_map(vars))
}

pub fn resolve_css_variables(css_string: &str, variables: &HashMap<String, String>) -> CssString {
    let var_regex = Regex::new(r"var\(--([^,)]+)(?:,\s*([^)]+))?\)").unwrap();

    let inner = var_regex
        .replace_all(css_string, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let fallback = caps.get(2).map(|m| m.as_str());

            // Look up the variable value
            if let Some(value) = variables.get(var_name) {
                value.clone()
            } else if let Some(fallback_value) = fallback {
                fallback_value.to_string()
            } else {
                // Keep original if no value found and no fallback
                caps[0].to_string()
            }
        })
        .to_string();

    CssString(inner)
}

// Helper function to build variable map from a more convenient format
pub fn build_variable_map(vars: &[(&str, &str)]) -> HashMap<String, String> {
    vars.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[derive(Debug)]
pub struct CssString(pub String);

impl IntoCssStr for CssString {
    fn get_style(&self) -> Result<String, std::io::Error> {
        Ok(self.0.clone())
    }
}
