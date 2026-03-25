use std::collections::HashMap;

pub(super) fn filter_aliases(
    tools: HashMap<String, HashMap<String, String>>,
    tool: Option<&str>,
) -> HashMap<String, HashMap<String, String>> {
    match tool {
        Some(name) => tools
            .get(name)
            .map(|aliases| {
                let mut map = HashMap::new();
                map.insert(name.to_string(), aliases.clone());
                map
            })
            .unwrap_or_default(),
        None => tools,
    }
}
