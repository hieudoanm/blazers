//! PocketBase-style filter expressions evaluated against JSON records.
//!
//! Grammar (subset of the PocketBase filter syntax):
//!   expr        := or
//!   or          := and ( '||' and )*
//!   and         := unary ( '&&' unary )*
//!   unary       := '!' unary | '(' expr ')' | comparison
//!   comparison  := operand (op operand)?
//!   operand     := literal | path
//!   literal     := 'string' | "string" | number | true | false | null
//!   path        := ident ('.' ident)* | '@' ident ('.' ident)*
//!
//! Paths resolve against a record view (`{id, created, updated, ...data}`)
//! with dot-notation into nested objects. `@request.auth.*` paths resolve
//! against the supplied auth context instead. Operators: `=`, `!=`, `>`,
//! `>=`, `<`, `<=`, `~` (case-insensitive contains), `!~`.

use serde_json::Value;

use parse::{Node, Operand};

mod parse;
mod tokenize;

/// Resolve a dotted path against `record`, falling back to `auth` for
/// `@request.*` prefixed paths.
fn resolve(path: &str, record: &Value, auth: &Value) -> Value {
    if let Some(rest) = path.strip_prefix("@request.") {
        let parts: Vec<&str> = rest.split('.').collect();
        // @request.auth.<field> and @request.data.<field>
        let (scope, field_path) = match parts.first() {
            Some(&"auth") => (auth, &parts[1..]),
            Some(&"data") => (record, &parts[1..]),
            _ => (auth, &parts[..]),
        };
        let mut cur = scope.clone();
        for part in field_path {
            cur = match cur.get(*part) {
                Some(v) => v.clone(),
                None => return Value::Null,
            };
        }
        return cur;
    }
    let mut cur = record.clone();
    for part in path.split('.') {
        cur = match cur.get(part) {
            Some(v) => v.clone(),
            None => return Value::Null,
        };
    }
    cur
}

fn operand_value(op: &Operand, record: &Value, auth: &Value) -> Value {
    match op {
        Operand::Lit(v) => v.clone(),
        Operand::Path(p) => resolve(p, record, auth),
    }
}

fn eval_node(node: &Node, record: &Value, auth: &Value) -> bool {
    match node {
        Node::Or(a, b) => eval_node(a, record, auth) || eval_node(b, record, auth),
        Node::And(a, b) => eval_node(a, record, auth) && eval_node(b, record, auth),
        Node::Not(a) => !eval_node(a, record, auth),
        Node::Truthy(op) => truthy(&operand_value(op, record, auth)),
        Node::Cmp(l, op, r) => compare(
            &operand_value(l, record, auth),
            op,
            &operand_value(r, record, auth),
        ),
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Numeric-aware equality: `42` matches `42.0`.
fn values_equal(l: &Value, r: &Value) -> bool {
    if l == r {
        return true;
    }
    match (l.as_f64(), r.as_f64()) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

fn compare(l: &Value, op: &str, r: &Value) -> bool {
    match op {
        "=" => values_equal(l, r),
        "!=" => !values_equal(l, r),
        "~" | "!~" => {
            let ls = l.as_str().unwrap_or("").to_lowercase();
            let rs = r.as_str().unwrap_or("").to_lowercase();
            let hit = ls.contains(&rs);
            if op == "~" { hit } else { !hit }
        }
        ">" | ">=" | "<" | "<=" => match (l.as_f64(), r.as_f64()) {
            (Some(a), Some(b)) => match op {
                ">" => a > b,
                ">=" => a >= b,
                "<" => a < b,
                _ => a <= b,
            },
            _ => {
                let (ls, rs) = (l.as_str().unwrap_or(""), r.as_str().unwrap_or(""));
                match op {
                    ">" => ls > rs,
                    ">=" => ls >= rs,
                    "<" => ls < rs,
                    _ => ls <= rs,
                }
            }
        },
        _ => false,
    }
}

/// Evaluate a filter expression against one record view. `auth` is the
/// `@request.auth` context (may be `Value::Null`).
pub fn eval(expr: &str, record: &Value, auth: &Value) -> Result<bool, String> {
    if expr.trim().is_empty() {
        return Ok(true);
    }
    let node = parse::parse(expr)?;
    Ok(eval_node(&node, record, auth))
}

/// Pre-parse a filter once for reuse across many records.
pub fn compile(expr: &str) -> Result<Filter, String> {
    Ok(Filter {
        node: if expr.trim().is_empty() {
            None
        } else {
            Some(parse::parse(expr)?)
        },
    })
}

pub struct Filter {
    node: Option<Node>,
}

impl Filter {
    pub fn matches(&self, record: &Value, auth: &Value) -> bool {
        match &self.node {
            None => true,
            Some(n) => eval_node(n, record, auth),
        }
    }
}

/// Sort key for a resolved path value: numbers first (numeric order), then
/// strings (lexicographic), then bools. `Null` sorts last in both directions.
pub fn sort_rank(value: &Value) -> (u8, String) {
    match value {
        Value::Null => (3, String::new()),
        Value::Number(n) => {
            // Order-preserving f64 → u64 bit transform, zero-padded so the
            // decimal string sorts lexicographically.
            let f = n.as_f64().unwrap_or(0.0);
            let bits = f.to_bits();
            let key = if bits & (1 << 63) != 0 {
                !bits
            } else {
                bits | (1 << 63)
            };
            (0, format!("{key:020}"))
        }
        Value::String(s) => (1, s.clone()),
        Value::Bool(b) => (2, b.to_string()),
        other => (1, other.to_string()),
    }
}

#[cfg(test)]
mod tests;
