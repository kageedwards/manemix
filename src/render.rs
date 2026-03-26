use std::collections::HashMap;
use tera::{Tera, Value, Result as TeraResult};

use crate::text;

/// Register custom Tera filters that replace ctemplate modifiers.
pub fn register_filters(tera: &mut Tera) {
    tera.register_filter("format_bbcode", filter_format);
    tera.register_filter("email_escape", filter_email);
    tera.register_filter("irc_escape", filter_irc);
}

fn filter_format(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(text::format_bbcode(s)))
}

fn filter_email(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(text::email_escape(s)))
}

fn filter_irc(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(text::irc_escape(s)))
}
