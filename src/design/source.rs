//! Source-level evidence. Syntax is observed; business meaning remains inferred.

use crate::ProjectMetrics;
use quote::ToTokens;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use syn::{spanned::Spanned, visit::Visit};

#[derive(Debug, Clone, Serialize)]
pub struct SourceItem {
    pub module: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    /// None denotes the analyzed working tree; deleted items retain their Git revision.
    pub revision: Option<String>,
    pub line: usize,
    pub end_line: usize,
    pub public: bool,
    pub is_test: bool,
    pub signature_types: Vec<String>,
    pub calls: Vec<String>,
    pub fields: Vec<(String, String)>,
    pub body_fingerprint: Option<String>,
    pub body_tokens: usize,
    pub forwards_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceFinding {
    pub kind: String,
    pub origin: String,
    pub items: Vec<SourceItem>,
    pub reason: String,
    pub unknowns: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SourceInventory {
    pub items: Vec<SourceItem>,
    pub findings: Vec<SourceFinding>,
    pub notes: Vec<String>,
    pub fingerprints: BTreeMap<String, String>,
}

/// Deterministic content identity, not a cryptographic integrity signature.
pub fn fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

impl SourceInventory {
    pub fn from_source(source: &str, path: &Path, module: &str, exclude_tests: bool) -> Self {
        let mut inventory = Self::default();
        inventory.scan(source, path, module, exclude_tests);
        inventory.detect_findings();
        inventory
    }

    pub fn read(metrics: &ProjectMetrics, exclude_tests: bool) -> Self {
        let mut inventory = Self::default();
        let mut files = BTreeMap::new();
        let mut modules: Vec<_> = metrics.modules.iter().collect();
        modules.sort_by_key(|(name, _)| *name);
        for (name, module) in modules {
            files.entry(module.path.clone()).or_insert(name.clone());
        }
        for (path, module) in files {
            match fs::read_to_string(&path) {
                Ok(source) => {
                    inventory
                        .fingerprints
                        .insert(path.display().to_string(), fingerprint(source.as_bytes()));
                    inventory.scan(&source, &path, &module, exclude_tests);
                }
                Err(error) => inventory.notes.push(format!("{}: {error}", path.display())),
            }
        }
        inventory
            .items
            .sort_by(|a, b| (&a.file_path, a.line, &a.name).cmp(&(&b.file_path, b.line, &b.name)));
        inventory.detect_findings();
        inventory
    }

    fn scan(&mut self, source: &str, path: &Path, module: &str, exclude_tests: bool) {
        match syn::parse_file(source) {
            Ok(file) => scan_items(
                &file.items,
                module,
                path,
                false,
                exclude_tests,
                &mut self.items,
            ),
            Err(error) => self.notes.push(format!("{}: {error}", path.display())),
        }
    }

    fn detect_findings(&mut self) {
        let mut bodies: BTreeMap<&str, Vec<&SourceItem>> = BTreeMap::new();
        let mut records: BTreeMap<&Vec<(String, String)>, Vec<&SourceItem>> = BTreeMap::new();
        for item in &self.items {
            if !item.is_test
                && item.body_tokens >= 24
                && let Some(hash) = &item.body_fingerprint
            {
                bodies.entry(hash).or_default().push(item);
            }
            if item.fields.len() >= 2 {
                records.entry(&item.fields).or_default().push(item);
            }
            if !item.is_test
                && let Some(target) = &item.forwards_to
            {
                self.findings.push(SourceFinding { kind: "forwarding-layer".into(), origin: "syntax".into(), items: vec![item.clone()], reason: format!("{} only forwards to {target}", item.name), unknowns: vec!["A stable name, visibility boundary, instrumentation point, or compatibility layer may justify forwarding.".into()] });
            }
        }
        for items in bodies.values().filter(|items| {
            items
                .iter()
                .map(|i| &i.module)
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        }) {
            self.findings.push(SourceFinding { kind: "identical-logic".into(), origin: "syntax".into(), items: items.iter().map(|i| (*i).clone()).collect(), reason: "Identical normalized Rust bodies occur in different modules.".into(), unknowns: vec!["Identical code does not establish a shared business rule; confirm whether requirements must change together.".into()] });
        }
        for items in records
            .values()
            .filter(|items| items.len() > 1 && items.iter().any(|i| i.public))
        {
            self.findings.push(SourceFinding { kind: "mirrored-record".into(), origin: "syntax".into(), items: items.iter().map(|i| (*i).clone()).collect(), reason: "Records expose the same named field types; the extra boundary may hide no model knowledge.".into(), unknowns: vec!["Separate ownership, validation, serialization or independent evolution can justify matching shapes.".into()] });
        }
        for item in self.items.iter().filter(|i| i.public && !i.is_test) {
            let leaked: Vec<_> = item
                .signature_types
                .iter()
                .filter(|ty| {
                    ty.contains("::internal::")
                        || self.items.iter().any(|definition| {
                            !definition.public
                                && definition.kind == "struct"
                                && type_mentions(ty, &definition.name)
                        })
                })
                .collect();
            if !leaked.is_empty() {
                self.findings.push(SourceFinding { kind: "implementation-type-exposure".into(), origin: "inferred".into(), items: vec![item.clone()], reason: format!("Public signature exposes implementation-shaped types: {}", leaked.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")), unknowns: vec!["Name matching is not Rust type checking; re-exports and opaque aliases need review.".into()] });
            }
        }
    }
}

pub fn type_mentions(ty: &str, name: &str) -> bool {
    ty.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|part| part == name)
}

fn test_attributes(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
            || attr.path().is_ident("cfg")
                && attr
                    .parse_args::<syn::Path>()
                    .is_ok_and(|path| path.is_ident("test"))
    })
}

fn scan_items(
    items: &[syn::Item],
    module: &str,
    path: &Path,
    inherited_test: bool,
    exclude_tests: bool,
    output: &mut Vec<SourceItem>,
) {
    let start = output.len();
    for item in items {
        match item {
            syn::Item::Mod(value) => {
                let is_test =
                    inherited_test || value.ident == "tests" || test_attributes(&value.attrs);
                if !(exclude_tests && is_test)
                    && let Some((_, children)) = &value.content
                {
                    scan_items(
                        children,
                        &format!("{module}::{}", value.ident),
                        path,
                        is_test,
                        exclude_tests,
                        output,
                    );
                }
            }
            syn::Item::Fn(value) => {
                let is_test = inherited_test || test_attributes(&value.attrs);
                if !(exclude_tests && is_test) {
                    output.push(function(
                        &value.sig,
                        Some(&value.block),
                        &value.vis,
                        module,
                        path,
                        is_test,
                        None,
                    ));
                }
            }
            syn::Item::Impl(value) => {
                if exclude_tests && (inherited_test || test_attributes(&value.attrs)) {
                    continue;
                }
                let parent = value.self_ty.to_token_stream().to_string();
                for child in &value.items {
                    if let syn::ImplItem::Fn(method) = child {
                        output.push(function(
                            &method.sig,
                            Some(&method.block),
                            &method.vis,
                            module,
                            path,
                            inherited_test,
                            Some(&parent),
                        ));
                    }
                }
            }
            syn::Item::Trait(value) => {
                if exclude_tests && (inherited_test || test_attributes(&value.attrs)) {
                    continue;
                }
                for child in &value.items {
                    if let syn::TraitItem::Fn(method) = child {
                        output.push(function(
                            &method.sig,
                            method.default.as_ref(),
                            &value.vis,
                            module,
                            path,
                            inherited_test,
                            Some(&value.ident.to_string()),
                        ));
                    }
                }
            }
            syn::Item::Struct(value) => {
                if exclude_tests && (inherited_test || test_attributes(&value.attrs)) {
                    continue;
                }
                let fields: Vec<_> = value
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        (
                            field
                                .ident
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| index.to_string()),
                            field.ty.to_token_stream().to_string(),
                        )
                    })
                    .collect();
                output.push(SourceItem {
                    module: module.into(),
                    name: value.ident.to_string(),
                    kind: "struct".into(),
                    file_path: path.display().to_string(),
                    revision: None,
                    line: value.span().start().line,
                    end_line: value.span().end().line,
                    public: matches!(value.vis, syn::Visibility::Public(_)),
                    is_test: inherited_test,
                    signature_types: fields.iter().map(|(_, ty)| ty.clone()).collect(),
                    fields,
                    calls: vec![],
                    body_fingerprint: None,
                    body_tokens: 0,
                    forwards_to: None,
                });
            }
            _ => {}
        }
    }
    let mut imports = BTreeMap::new();
    for item in items {
        if let syn::Item::Use(import) = item {
            collect_imports(&import.tree, "", &mut imports);
        }
    }
    for item in output[start..]
        .iter_mut()
        .filter(|item| item.module == module)
    {
        for ty in &mut item.signature_types {
            *ty = ty
                .split_whitespace()
                .map(|word| imports.get(word).map(String::as_str).unwrap_or(word))
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
}

fn collect_imports(tree: &syn::UseTree, prefix: &str, imports: &mut BTreeMap<String, String>) {
    match tree {
        syn::UseTree::Path(path) => {
            collect_imports(&path.tree, &format!("{prefix}{}::", path.ident), imports)
        }
        syn::UseTree::Name(name) => {
            imports.insert(name.ident.to_string(), format!("{prefix}{}", name.ident));
        }
        syn::UseTree::Rename(name) => {
            imports.insert(name.rename.to_string(), format!("{prefix}{}", name.ident));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_imports(item, prefix, imports);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn function(
    sig: &syn::Signature,
    body: Option<&syn::Block>,
    visibility: &syn::Visibility,
    module: &str,
    path: &Path,
    is_test: bool,
    parent: Option<&str>,
) -> SourceItem {
    let name = parent
        .map(|p| format!("{p}::{}", sig.ident))
        .unwrap_or_else(|| sig.ident.to_string());
    let signature_types = sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(value) = arg {
                Some(value.ty.to_token_stream().to_string())
            } else {
                None
            }
        })
        .chain(match &sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
        })
        .collect();
    let text = body.map(|b| b.to_token_stream().to_string());
    let mut calls = Calls::default();
    if let Some(body) = body {
        calls.visit_block(body);
    }
    let parameters: BTreeSet<_> = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(arg) => {
                if let syn::Pat::Ident(pattern) = &*arg.pat {
                    Some(pattern.ident.to_string())
                } else {
                    None
                }
            }
            syn::FnArg::Receiver(_) => Some("self".into()),
        })
        .collect();
    let forwards_to = body.and_then(|block| match block.stmts.as_slice() {
        [syn::Stmt::Expr(expression, _)] => forwarded_call(expression, &parameters),
        _ => None,
    });
    SourceItem {
        module: module.into(),
        name,
        kind: if parent.is_some() {
            "method"
        } else {
            "function"
        }
        .into(),
        file_path: path.display().to_string(),
        revision: None,
        line: sig.span().start().line,
        end_line: body
            .map(|b| b.span().end().line)
            .unwrap_or_else(|| sig.span().end().line),
        public: matches!(visibility, syn::Visibility::Public(_)),
        is_test,
        signature_types,
        calls: calls.names.into_iter().collect(),
        fields: vec![],
        body_fingerprint: text.as_ref().map(|text| fingerprint(text.as_bytes())),
        body_tokens: text
            .as_deref()
            .map(|text| text.split_whitespace().count())
            .unwrap_or(0),
        forwards_to,
    }
}

fn forwarded_call(expression: &syn::Expr, parameters: &BTreeSet<String>) -> Option<String> {
    match expression {
        syn::Expr::Call(call)
            if call
                .args
                .iter()
                .all(|arg| forwarded_argument(arg, parameters)) =>
        {
            Some(call.func.to_token_stream().to_string())
        }
        syn::Expr::MethodCall(call)
            if forwarded_argument(&call.receiver, parameters)
                && call
                    .args
                    .iter()
                    .all(|arg| forwarded_argument(arg, parameters)) =>
        {
            Some(format!(
                "{}.{}",
                call.receiver.to_token_stream(),
                call.method
            ))
        }
        syn::Expr::Return(value) => value
            .expr
            .as_deref()
            .and_then(|expr| forwarded_call(expr, parameters)),
        syn::Expr::Try(value) => forwarded_call(&value.expr, parameters),
        syn::Expr::Await(value) => forwarded_call(&value.base, parameters),
        _ => None,
    }
}

fn forwarded_argument(expression: &syn::Expr, parameters: &BTreeSet<String>) -> bool {
    match expression {
        syn::Expr::Path(path) => path
            .path
            .get_ident()
            .is_some_and(|name| parameters.contains(&name.to_string())),
        syn::Expr::Reference(reference) => forwarded_argument(&reference.expr, parameters),
        syn::Expr::Field(field) => forwarded_argument(&field.base, parameters),
        _ => false,
    }
}

#[derive(Default)]
struct Calls {
    names: BTreeSet<String>,
}
impl<'ast> Visit<'ast> for Calls {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.names
            .insert(node.func.to_token_stream().to_string().replace(' ', ""));
        syn::visit::visit_expr_call(self, node);
    }
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.names.insert(node.method.to_string());
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_signature_imports_and_keeps_computation_out_of_forwarding() {
        let inventory = SourceInventory::from_source(
            "use external::Client as Http;\npub fn client() -> Http { create() }\nfn work(n:u32) { send(n + 1) }\n#[cfg(feature=\"latest\")] pub fn normal() {}",
            Path::new("src/lib.rs"),
            "lib",
            true,
        );
        assert!(
            inventory.items[0]
                .signature_types
                .iter()
                .any(|ty| ty.contains("external::Client"))
        );
        assert!(inventory.items[1].forwards_to.is_none());
        assert_eq!(inventory.items.len(), 3);
    }
    #[test]
    fn records_real_spans_and_distinguishes_forwarding_from_logic() {
        let mut inventory = SourceInventory::default();
        inventory.scan(
            "\npub fn forward(n:u32)->u32 { other(n) }\nfn actual(n:u32)->u32 { n+1 }",
            Path::new("src/lib.rs"),
            "lib",
            false,
        );
        assert_eq!(inventory.items[0].line, 2);
        assert_eq!(inventory.items[0].forwards_to.as_deref(), Some("other"));
        assert!(inventory.items[1].forwards_to.is_none());
    }
}
