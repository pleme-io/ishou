//! JSON renderer — emits W3C Design Tokens-style JSON so Figma / Style
//! Dictionary / downstream tools can import.

use ishou_tokens::TokenSet;
use serde_json::{Map, Value, json};

pub fn render(t: &TokenSet) -> String {
    let mut root = Map::new();
    // colors
    let mut color = Map::new();
    for (k, v) in t.color.entries() {
        color.insert(
            k.replace('_', "-"),
            json!({ "$type": "color", "$value": v.hex() }),
        );
    }
    root.insert("color".into(), Value::Object(color));

    // spacing
    let mut spacing = Map::new();
    for (k, v) in t.spacing.pairs() {
        spacing.insert(
            k.into(),
            json!({ "$type": "dimension", "$value": format!("{v}px") }),
        );
    }
    root.insert("spacing".into(), Value::Object(spacing));

    // radius
    let mut radius = Map::new();
    for (k, v) in t.radius.pairs() {
        radius.insert(
            k.into(),
            json!({ "$type": "dimension", "$value": format!("{v}px") }),
        );
    }
    root.insert("radius".into(), Value::Object(radius));

    // fontFamily
    let f = &t.typography.families;
    root.insert(
        "fontFamily".into(),
        json!({
            "serif":   { "$type": "fontFamily", "$value": f.serif },
            "sans":    { "$type": "fontFamily", "$value": f.sans },
            "mono":    { "$type": "fontFamily", "$value": f.mono },
            "display": { "$type": "fontFamily", "$value": f.display },
        }),
    );

    serde_json::to_string_pretty(&root).unwrap()
}
