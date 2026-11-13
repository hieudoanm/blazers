//! Filter expression parser (internal to `filter`).

use serde_json::Value;

use super::tokenize::{Tok, tokenize};

#[derive(Debug)]
pub(super) enum Node {
    Or(Box<Node>, Box<Node>),
    And(Box<Node>, Box<Node>),
    Not(Box<Node>),
    Cmp(Operand, &'static str, Operand),
    Truthy(Operand),
}

#[derive(Debug, Clone)]
pub(super) enum Operand {
    Lit(Value),
    Path(String),
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }
    fn next(&mut self) -> Tok {
        let t = self.toks[self.pos.min(self.toks.len() - 1)].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn parse_expr(&mut self) -> Result<Node, String> {
        self.parse_or()
    }
    fn parse_or(&mut self) -> Result<Node, String> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Tok::Or) {
            self.next();
            let right = self.parse_and()?;
            left = Node::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn parse_and(&mut self) -> Result<Node, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek(), Tok::And) {
            self.next();
            let right = self.parse_unary()?;
            left = Node::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn parse_unary(&mut self) -> Result<Node, String> {
        match self.peek().clone() {
            Tok::Not => {
                self.next();
                Ok(Node::Not(Box::new(self.parse_unary()?)))
            }
            Tok::LParen => {
                self.next();
                let inner = self.parse_expr()?;
                match self.next() {
                    Tok::RParen => Ok(inner),
                    _ => Err("expected ')'".into()),
                }
            }
            _ => self.parse_comparison(),
        }
    }
    fn parse_comparison(&mut self) -> Result<Node, String> {
        let left = self.parse_operand()?;
        match self.peek().clone() {
            Tok::Op(op) => {
                self.next();
                let right = self.parse_operand()?;
                Ok(Node::Cmp(left, op, right))
            }
            _ => Ok(Node::Truthy(left)),
        }
    }
    fn parse_operand(&mut self) -> Result<Operand, String> {
        match self.next() {
            Tok::Str(s) => Ok(Operand::Lit(Value::String(s))),
            Tok::Num(n) => Ok(Operand::Lit(serde_json::json!(n))),
            Tok::Bool(b) => Ok(Operand::Lit(Value::Bool(b))),
            Tok::Null => Ok(Operand::Lit(Value::Null)),
            Tok::Ident(w) => Ok(Operand::Path(w)),
            other => Err(format!("expected operand, got {other:?}")),
        }
    }
}

pub(super) fn parse(input: &str) -> Result<Node, String> {
    let toks = tokenize(input)?;
    let mut p = Parser { toks, pos: 0 };
    let node = p.parse_expr()?;
    if !matches!(p.peek(), Tok::Eof) {
        return Err("unexpected trailing input".into());
    }
    Ok(node)
}
