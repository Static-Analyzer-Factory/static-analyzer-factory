use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use proc_macro2::Span;
use quote::ToTokens;
use serde::Serialize;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Block, Expr, ExprCall, ExprMethodCall, FnArg, ImplItem, ImplItemFn, Item, ItemFn, Pat, Path,
    Signature,
};

#[derive(Debug, Serialize)]
struct FactFile {
    schema: &'static str,
    source: String,
    functions: Vec<FunctionFact>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FunctionFact {
    name: String,
    params: Vec<String>,
    range: SourceRange,
    hash: String,
    calls: Vec<String>,
    source_hints: Vec<String>,
    sink_hints: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceRange {
    start_line: usize,
    end_line: usize,
}

#[derive(Default)]
struct CallCollector {
    calls: BTreeSet<String>,
    source_hints: BTreeSet<String>,
    sink_hints: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for CallCollector {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Some(name) = expr_name(&node.func) {
            self.record_call(name);
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.record_call(node.method.to_string());
        visit::visit_expr_method_call(self, node);
    }
}

impl CallCollector {
    fn record_call(&mut self, name: String) {
        if matches!(name.as_str(), "env::args" | "env::var" | "getenv")
            || name.ends_with("::env::args")
            || name.ends_with("::env::var")
        {
            self.source_hints.insert(name.clone());
        }
        if matches!(name.as_str(), "Command::new" | "system") || name.ends_with("::Command::new") {
            self.sink_hints.insert(name.clone());
        }
        self.calls.insert(name);
    }
}

fn expr_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) => Some(path_to_string(&path.path)),
        _ => None,
    }
}

fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn function_params(sig: &Signature) -> Vec<String> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(typed) => match typed.pat.as_ref() {
                Pat::Ident(ident) => Some(ident.ident.to_string()),
                _ => None,
            },
            FnArg::Receiver(_) => Some("self".to_string()),
        })
        .collect()
}

fn span_range(span: Span) -> SourceRange {
    let start = span.start();
    let end = span.end();
    SourceRange {
        start_line: start.line,
        end_line: end.line,
    }
}

fn hash_tokens(tokens: impl ToTokens) -> String {
    let text = tokens.to_token_stream().to_string();
    blake3::hash(text.as_bytes()).to_hex().as_str()[..16].to_string()
}

fn function_fact(item: &ItemFn) -> FunctionFact {
    function_fact_parts(&item.sig, &item.block, item.span(), hash_tokens(item))
}

fn impl_method_fact(owner: &str, item: &ImplItemFn) -> FunctionFact {
    let mut fact = function_fact_parts(&item.sig, &item.block, item.span(), hash_tokens(item));
    fact.name = format!("{owner}::{}", fact.name);
    fact
}

fn function_fact_parts(
    sig: &Signature,
    block: &Block,
    span: proc_macro2::Span,
    hash: String,
) -> FunctionFact {
    let mut collector = CallCollector::default();
    collector.visit_block(block);

    FunctionFact {
        name: sig.ident.to_string(),
        params: function_params(sig),
        range: span_range(span),
        hash,
        calls: collector.calls.into_iter().collect(),
        source_hints: collector.source_hints.into_iter().collect(),
        sink_hints: collector.sink_hints.into_iter().collect(),
    }
}

fn main() -> Result<()> {
    let input = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: livesaferust-syn-sidecar <file.rs>")?;
    let source = std::fs::read_to_string(&input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let file =
        syn::parse_file(&source).with_context(|| format!("failed to parse {}", input.display()))?;

    let mut functions = Vec::new();
    for item in &file.items {
        match item {
            Item::Fn(function) => functions.push(function_fact(function)),
            Item::Impl(impl_block) => {
                let owner = impl_block.self_ty.to_token_stream().to_string();
                for item in &impl_block.items {
                    if let ImplItem::Fn(method) = item {
                        functions.push(impl_method_fact(&owner, method));
                    }
                }
            }
            _ => {}
        }
    }

    let facts = FactFile {
        schema: "livesaferust.syn-sidecar/0.1",
        source: input.display().to_string(),
        functions,
    };

    println!("{}", serde_json::to_string_pretty(&facts)?);
    Ok(())
}
