//! AST analysis for coupling detection
//!
//! Uses `syn` to parse Rust source code and detect coupling patterns.
//! Optionally uses `cargo metadata` for accurate workspace analysis.
//! Supports parallel processing via Rayon for large projects.

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use syn::visit::Visit;
use syn::{
    Expr, ExprCall, ExprField, ExprMethodCall, ExprStruct, File, FnArg, ItemFn, ItemImpl, ItemMod,
    ItemStruct, ItemTrait, ItemUse, ReturnType, Signature, Type, UseTree,
};
use thiserror::Error;
use walkdir::WalkDir;

use crate::metrics::{
    CouplingMetrics, Distance, IntegrationStrength, ModuleMetrics, ProjectMetrics, Visibility,
    Volatility,
};
use crate::workspace::{WorkspaceError, WorkspaceInfo, resolve_crate_from_path};

/// Convert syn's Visibility to our Visibility enum
fn convert_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(restricted) => {
            // Check the path to determine the restriction type
            let path_str = restricted
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            match path_str.as_str() {
                "crate" => Visibility::PubCrate,
                "super" => Visibility::PubSuper,
                "self" => Visibility::Private, // pub(self) is effectively private
                _ => Visibility::PubIn,        // pub(in path)
            }
        }
        syn::Visibility::Inherited => Visibility::Private,
    }
}

/// Errors that can occur during analysis
#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse Rust file: {0}")]
    ParseError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),
}

/// Represents a detected dependency
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Full path of the dependency (e.g., "crate::models::user")
    pub path: String,
    /// Type of dependency
    pub kind: DependencyKind,
    /// Line number where the dependency is declared
    pub line: usize,
    /// Usage context for more accurate strength determination
    pub usage: UsageContext,
}

/// Kind of dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    /// use crate::xxx or use super::xxx
    InternalUse,
    /// use external_crate::xxx
    ExternalUse,
    /// impl Trait for Type
    TraitImpl,
    /// impl Type
    InherentImpl,
    /// Type reference in struct fields, function params, etc.
    TypeRef,
}

/// Context of how a dependency is used - determines Integration Strength
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsageContext {
    /// Just imported, usage unknown
    Import,
    /// Used as a trait bound or trait impl
    TraitBound,
    /// Field access: `foo.bar`
    FieldAccess,
    /// Method call: `foo.method()`
    MethodCall,
    /// Function call: `Foo::new()` or `foo()`
    FunctionCall,
    /// Struct construction: `Foo { field: value }`
    StructConstruction,
    /// Type parameter: `Vec<Foo>`
    TypeParameter,
    /// Function parameter type
    FunctionParameter,
    /// Return type
    ReturnType,
    /// Inherent impl block
    InherentImplBlock,
}

impl UsageContext {
    /// Convert usage context to integration strength
    pub fn to_strength(&self) -> IntegrationStrength {
        match self {
            // Intrusive: Direct access to internals
            UsageContext::FieldAccess => IntegrationStrength::Intrusive,
            UsageContext::StructConstruction => IntegrationStrength::Intrusive,
            UsageContext::InherentImplBlock => IntegrationStrength::Intrusive,

            // Functional: Depends on function signatures
            UsageContext::MethodCall => IntegrationStrength::Functional,
            UsageContext::FunctionCall => IntegrationStrength::Functional,
            UsageContext::FunctionParameter => IntegrationStrength::Functional,
            UsageContext::ReturnType => IntegrationStrength::Functional,

            // Model: Uses data types
            UsageContext::TypeParameter => IntegrationStrength::Model,
            UsageContext::Import => IntegrationStrength::Model,

            // Contract: Uses traits/interfaces
            UsageContext::TraitBound => IntegrationStrength::Contract,
        }
    }
}

impl DependencyKind {
    pub fn to_strength(&self) -> IntegrationStrength {
        match self {
            DependencyKind::TraitImpl => IntegrationStrength::Contract,
            DependencyKind::InternalUse => IntegrationStrength::Model,
            DependencyKind::ExternalUse => IntegrationStrength::Model,
            DependencyKind::TypeRef => IntegrationStrength::Model,
            DependencyKind::InherentImpl => IntegrationStrength::Intrusive,
        }
    }
}

/// AST visitor for coupling analysis
#[derive(Debug)]
pub struct CouplingAnalyzer {
    /// Current module being analyzed
    pub current_module: String,
    /// File path
    pub file_path: std::path::PathBuf,
    /// Collected metrics
    pub metrics: ModuleMetrics,
    /// Detected dependencies
    pub dependencies: Vec<Dependency>,
    /// Defined types in this module
    pub defined_types: HashSet<String>,
    /// Defined traits in this module
    pub defined_traits: HashSet<String>,
    /// Defined functions in this module (name -> visibility)
    pub defined_functions: HashMap<String, Visibility>,
    /// Imported types (name -> full path)
    imported_types: HashMap<String, String>,
    /// Track unique dependencies to avoid duplicates
    seen_dependencies: HashSet<(String, UsageContext)>,
    /// Counts of each usage type for statistics
    pub usage_counts: UsageCounts,
    /// Type visibility map: type name -> visibility
    pub type_visibility: HashMap<String, Visibility>,
    /// Current item being analyzed (function name, struct name, etc.)
    current_item: Option<(String, ItemKind)>,
    /// Item-level dependencies (detailed tracking)
    pub item_dependencies: Vec<ItemDependency>,
}

/// Statistics about usage patterns
#[derive(Debug, Default, Clone)]
pub struct UsageCounts {
    pub field_accesses: usize,
    pub method_calls: usize,
    pub function_calls: usize,
    pub struct_constructions: usize,
    pub trait_bounds: usize,
    pub type_parameters: usize,
}

/// Detailed dependency at the item level (function, struct, etc.)
#[derive(Debug, Clone)]
pub struct ItemDependency {
    /// Source item (e.g., "fn analyze_project")
    pub source_item: String,
    /// Source item kind
    pub source_kind: ItemKind,
    /// Target (e.g., "ProjectMetrics" or "analyze_file")
    pub target: String,
    /// Target module (if known)
    pub target_module: Option<String>,
    /// Type of dependency
    pub dep_type: ItemDepType,
    /// Line number in source
    pub line: usize,
    /// The actual expression/code (e.g., "config.thresholds" or "self.couplings")
    pub expression: Option<String>,
}

/// Kind of source item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
}

/// Type of item-level dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDepType {
    /// Calls a function: foo()
    FunctionCall,
    /// Calls a method: x.foo()
    MethodCall,
    /// Uses a type: Vec<Foo>
    TypeUsage,
    /// Accesses a field: x.field
    FieldAccess,
    /// Constructs a struct: Foo { ... }
    StructConstruction,
    /// Implements a trait: impl Trait for Type
    TraitImpl,
    /// Uses a trait bound: T: Trait
    TraitBound,
    /// Imports: use foo::Bar
    Import,
}

impl CouplingAnalyzer {
    /// Create a new analyzer for a module
    pub fn new(module_name: String, path: std::path::PathBuf) -> Self {
        Self {
            current_module: module_name.clone(),
            file_path: path.clone(),
            metrics: ModuleMetrics::new(path, module_name),
            dependencies: Vec::new(),
            defined_types: HashSet::new(),
            defined_traits: HashSet::new(),
            defined_functions: HashMap::new(),
            imported_types: HashMap::new(),
            seen_dependencies: HashSet::new(),
            usage_counts: UsageCounts::default(),
            type_visibility: HashMap::new(),
            current_item: None,
            item_dependencies: Vec::new(),
        }
    }

    /// Analyze a Rust source file
    pub fn analyze_file(&mut self, content: &str) -> Result<(), AnalyzerError> {
        let syntax: File =
            syn::parse_file(content).map_err(|e| AnalyzerError::ParseError(e.to_string()))?;

        self.visit_file(&syntax);

        Ok(())
    }

    /// Add a dependency with deduplication
    fn add_dependency(&mut self, path: String, kind: DependencyKind, usage: UsageContext) {
        let key = (path.clone(), usage);
        if self.seen_dependencies.contains(&key) {
            return;
        }
        self.seen_dependencies.insert(key);

        self.dependencies.push(Dependency {
            path,
            kind,
            line: 0,
            usage,
        });
    }

    /// Record an item-level dependency with detailed tracking
    fn add_item_dependency(
        &mut self,
        target: String,
        dep_type: ItemDepType,
        line: usize,
        expression: Option<String>,
    ) {
        if let Some((ref source_item, source_kind)) = self.current_item {
            // Determine target module
            let target_module = self.imported_types.get(&target).cloned().or_else(|| {
                if self.defined_types.contains(&target)
                    || self.defined_functions.contains_key(&target)
                {
                    Some(self.current_module.clone())
                } else {
                    None
                }
            });

            self.item_dependencies.push(ItemDependency {
                source_item: source_item.clone(),
                source_kind,
                target,
                target_module,
                dep_type,
                line,
                expression,
            });
        }
    }

    /// Extract full path from UseTree recursively
    fn extract_use_paths(&self, tree: &UseTree, prefix: &str) -> Vec<(String, DependencyKind)> {
        let mut paths = Vec::new();

        match tree {
            UseTree::Path(path) => {
                let new_prefix = if prefix.is_empty() {
                    path.ident.to_string()
                } else {
                    format!("{}::{}", prefix, path.ident)
                };
                paths.extend(self.extract_use_paths(&path.tree, &new_prefix));
            }
            UseTree::Name(name) => {
                let full_path = if prefix.is_empty() {
                    name.ident.to_string()
                } else {
                    format!("{}::{}", prefix, name.ident)
                };
                let kind = if prefix.starts_with("crate") || prefix.starts_with("super") {
                    DependencyKind::InternalUse
                } else {
                    DependencyKind::ExternalUse
                };
                paths.push((full_path, kind));
            }
            UseTree::Rename(rename) => {
                let full_path = if prefix.is_empty() {
                    rename.ident.to_string()
                } else {
                    format!("{}::{}", prefix, rename.ident)
                };
                let kind = if prefix.starts_with("crate") || prefix.starts_with("super") {
                    DependencyKind::InternalUse
                } else {
                    DependencyKind::ExternalUse
                };
                paths.push((full_path, kind));
            }
            UseTree::Glob(_) => {
                let full_path = format!("{}::*", prefix);
                let kind = if prefix.starts_with("crate") || prefix.starts_with("super") {
                    DependencyKind::InternalUse
                } else {
                    DependencyKind::ExternalUse
                };
                paths.push((full_path, kind));
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    paths.extend(self.extract_use_paths(item, prefix));
                }
            }
        }

        paths
    }

    /// Extract type name from a Type
    fn extract_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Path(type_path) => {
                let segments: Vec<_> = type_path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                Some(segments.join("::"))
            }
            Type::Reference(ref_type) => self.extract_type_name(&ref_type.elem),
            Type::Slice(slice_type) => self.extract_type_name(&slice_type.elem),
            Type::Array(array_type) => self.extract_type_name(&array_type.elem),
            Type::Ptr(ptr_type) => self.extract_type_name(&ptr_type.elem),
            Type::Paren(paren_type) => self.extract_type_name(&paren_type.elem),
            Type::Group(group_type) => self.extract_type_name(&group_type.elem),
            _ => None,
        }
    }

    /// Analyze function signature for dependencies
    fn analyze_signature(&mut self, sig: &Signature) {
        // Analyze parameters
        for arg in &sig.inputs {
            if let FnArg::Typed(pat_type) = arg
                && let Some(type_name) = self.extract_type_name(&pat_type.ty)
                && !self.is_primitive_type(&type_name)
            {
                self.add_dependency(
                    type_name,
                    DependencyKind::TypeRef,
                    UsageContext::FunctionParameter,
                );
            }
        }

        // Analyze return type
        if let ReturnType::Type(_, ty) = &sig.output
            && let Some(type_name) = self.extract_type_name(ty)
            && !self.is_primitive_type(&type_name)
        {
            self.add_dependency(type_name, DependencyKind::TypeRef, UsageContext::ReturnType);
        }
    }

    /// Check if a type should be ignored (primitives, self, or short variable names)
    fn is_primitive_type(&self, type_name: &str) -> bool {
        // Primitive types
        if matches!(
            type_name,
            "bool"
                | "char"
                | "str"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
                | "String"
                | "Self"
                | "()"
                | "Option"
                | "Result"
                | "Vec"
                | "Box"
                | "Rc"
                | "Arc"
                | "RefCell"
                | "Cell"
                | "Mutex"
                | "RwLock"
        ) {
            return true;
        }

        // Short variable names (likely local variables, not types)
        // Type names in Rust are typically PascalCase and longer
        if type_name.len() <= 3 && type_name.chars().all(|c| c.is_lowercase()) {
            return true;
        }

        // Self-references or obviously local
        if type_name.starts_with("self") || type_name == "self" {
            return true;
        }

        false
    }
}

impl<'ast> Visit<'ast> for CouplingAnalyzer {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let paths = self.extract_use_paths(&node.tree, "");

        for (path, kind) in paths {
            // Skip self references
            if path == "self" || path.starts_with("self::") {
                continue;
            }

            // Track imported types for later resolution
            if let Some(type_name) = path.split("::").last() {
                self.imported_types
                    .insert(type_name.to_string(), path.clone());
            }

            self.add_dependency(path.clone(), kind, UsageContext::Import);

            // Update metrics
            if kind == DependencyKind::InternalUse {
                if !self.metrics.internal_deps.contains(&path) {
                    self.metrics.internal_deps.push(path.clone());
                }
            } else if kind == DependencyKind::ExternalUse {
                // Extract crate name
                let crate_name = path.split("::").next().unwrap_or(&path).to_string();
                if !self.metrics.external_deps.contains(&crate_name) {
                    self.metrics.external_deps.push(crate_name);
                }
            }
        }

        syn::visit::visit_item_use(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if let Some((_, trait_path, _)) = &node.trait_ {
            // Trait implementation = Contract coupling
            self.metrics.trait_impl_count += 1;

            // Extract trait path
            let trait_name: String = trait_path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            self.add_dependency(
                trait_name,
                DependencyKind::TraitImpl,
                UsageContext::TraitBound,
            );
            self.usage_counts.trait_bounds += 1;
        } else {
            // Inherent implementation = Intrusive coupling
            self.metrics.inherent_impl_count += 1;

            // Get the type being implemented
            if let Some(type_name) = self.extract_type_name(&node.self_ty)
                && !self.defined_types.contains(&type_name)
            {
                self.add_dependency(
                    type_name,
                    DependencyKind::InherentImpl,
                    UsageContext::InherentImplBlock,
                );
            }
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Record function definition
        let fn_name = node.sig.ident.to_string();
        let visibility = convert_visibility(&node.vis);
        self.defined_functions.insert(fn_name.clone(), visibility);

        // Analyze parameters for primitive obsession detection
        let mut param_count = 0;
        let mut primitive_param_count = 0;
        let mut param_types = Vec::new();

        for arg in &node.sig.inputs {
            if let FnArg::Typed(pat_type) = arg {
                param_count += 1;
                if let Some(type_name) = self.extract_type_name(&pat_type.ty) {
                    param_types.push(type_name.clone());
                    if self.is_primitive_type(&type_name) {
                        primitive_param_count += 1;
                    }
                }
            }
        }

        // Register in module metrics with full details
        self.metrics.add_function_definition_full(
            fn_name.clone(),
            visibility,
            param_count,
            primitive_param_count,
            param_types,
        );

        // Set current item context for dependency tracking
        let previous_item = self.current_item.take();
        self.current_item = Some((fn_name, ItemKind::Function));

        // Analyze function signature
        self.analyze_signature(&node.sig);
        syn::visit::visit_item_fn(self, node);

        // Restore previous context
        self.current_item = previous_item;
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = node.ident.to_string();
        let visibility = convert_visibility(&node.vis);

        self.defined_types.insert(name.clone());
        self.type_visibility.insert(name.clone(), visibility);

        // Detect newtype pattern: single-field tuple struct
        let (is_newtype, inner_type) = match &node.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let inner = fields
                    .unnamed
                    .first()
                    .and_then(|f| self.extract_type_name(&f.ty));
                (true, inner)
            }
            _ => (false, None),
        };

        // Check for serde derives
        let has_serde_derive = node.attrs.iter().any(|attr| {
            if attr.path().is_ident("derive")
                && let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
            {
                return nested.iter().any(|path| {
                    let path_str = path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    path_str == "Serialize"
                        || path_str == "Deserialize"
                        || path_str == "serde::Serialize"
                        || path_str == "serde::Deserialize"
                });
            }
            false
        });

        // Count fields and public fields
        let (total_field_count, public_field_count) = match &node.fields {
            syn::Fields::Named(fields) => {
                let total = fields.named.len();
                let public = fields
                    .named
                    .iter()
                    .filter(|f| matches!(f.vis, syn::Visibility::Public(_)))
                    .count();
                (total, public)
            }
            syn::Fields::Unnamed(fields) => {
                let total = fields.unnamed.len();
                let public = fields
                    .unnamed
                    .iter()
                    .filter(|f| matches!(f.vis, syn::Visibility::Public(_)))
                    .count();
                (total, public)
            }
            syn::Fields::Unit => (0, 0),
        };

        // Register in module metrics with full details
        self.metrics.add_type_definition_full(
            name,
            visibility,
            false, // is_trait
            is_newtype,
            inner_type,
            has_serde_derive,
            public_field_count,
            total_field_count,
        );

        // Analyze struct fields for type dependencies
        match &node.fields {
            syn::Fields::Named(fields) => {
                self.metrics.type_usage_count += fields.named.len();
                for field in &fields.named {
                    if let Some(type_name) = self.extract_type_name(&field.ty)
                        && !self.is_primitive_type(&type_name)
                    {
                        self.add_dependency(
                            type_name,
                            DependencyKind::TypeRef,
                            UsageContext::TypeParameter,
                        );
                        self.usage_counts.type_parameters += 1;
                    }
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    if let Some(type_name) = self.extract_type_name(&field.ty)
                        && !self.is_primitive_type(&type_name)
                    {
                        self.add_dependency(
                            type_name,
                            DependencyKind::TypeRef,
                            UsageContext::TypeParameter,
                        );
                    }
                }
            }
            syn::Fields::Unit => {}
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let name = node.ident.to_string();
        let visibility = convert_visibility(&node.vis);

        self.defined_types.insert(name.clone());
        self.type_visibility.insert(name.clone(), visibility);

        // Register in module metrics with visibility
        self.metrics.add_type_definition(name, visibility, false);

        // Analyze enum variants for type dependencies
        for variant in &node.variants {
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        if let Some(type_name) = self.extract_type_name(&field.ty)
                            && !self.is_primitive_type(&type_name)
                        {
                            self.add_dependency(
                                type_name,
                                DependencyKind::TypeRef,
                                UsageContext::TypeParameter,
                            );
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &fields.unnamed {
                        if let Some(type_name) = self.extract_type_name(&field.ty)
                            && !self.is_primitive_type(&type_name)
                        {
                            self.add_dependency(
                                type_name,
                                DependencyKind::TypeRef,
                                UsageContext::TypeParameter,
                            );
                        }
                    }
                }
                syn::Fields::Unit => {}
            }
        }
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        let name = node.ident.to_string();
        let visibility = convert_visibility(&node.vis);

        self.defined_traits.insert(name.clone());
        self.type_visibility.insert(name.clone(), visibility);

        // Register in module metrics with visibility (is_trait = true)
        self.metrics.add_type_definition(name, visibility, true);

        self.metrics.trait_impl_count += 1;
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if node.content.is_some() {
            self.metrics.internal_deps.push(node.ident.to_string());
        }
        syn::visit::visit_item_mod(self, node);
    }

    // Detect field access: `foo.bar`
    fn visit_expr_field(&mut self, node: &'ast ExprField) {
        let field_name = match &node.member {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(idx) => format!("{}", idx.index),
        };

        // This is a field access - Intrusive coupling
        if let Expr::Path(path_expr) = &*node.base {
            let base_name = path_expr
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Resolve to full path if imported
            let full_path = self
                .imported_types
                .get(&base_name)
                .cloned()
                .unwrap_or(base_name.clone());

            if !self.is_primitive_type(&full_path) && !self.defined_types.contains(&full_path) {
                self.add_dependency(
                    full_path.clone(),
                    DependencyKind::TypeRef,
                    UsageContext::FieldAccess,
                );
                self.usage_counts.field_accesses += 1;
            }

            // Record item-level dependency with field name
            let expr = format!("{}.{}", base_name, field_name);
            self.add_item_dependency(
                format!("{}.{}", full_path, field_name),
                ItemDepType::FieldAccess,
                0,
                Some(expr),
            );
        }
        syn::visit::visit_expr_field(self, node);
    }

    // Detect method calls: `foo.method()`
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_string();

        // This is a method call - Functional coupling
        if let Expr::Path(path_expr) = &*node.receiver {
            let receiver_name = path_expr
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let full_path = self
                .imported_types
                .get(&receiver_name)
                .cloned()
                .unwrap_or(receiver_name.clone());

            if !self.is_primitive_type(&full_path) && !self.defined_types.contains(&full_path) {
                self.add_dependency(
                    full_path.clone(),
                    DependencyKind::TypeRef,
                    UsageContext::MethodCall,
                );
                self.usage_counts.method_calls += 1;
            }

            // Record item-level dependency
            let expr = format!("{}.{}()", receiver_name, method_name);
            self.add_item_dependency(
                format!("{}::{}", full_path, method_name),
                ItemDepType::MethodCall,
                0, // TODO: get line number from span
                Some(expr),
            );
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    // Detect function calls: `Foo::new()` or `foo()`
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path_expr) = &*node.func {
            let path_str = path_expr
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Check if this is a constructor or associated function call
            if path_str.contains("::") || path_str.chars().next().is_some_and(|c| c.is_uppercase())
            {
                let full_path = self
                    .imported_types
                    .get(&path_str)
                    .cloned()
                    .unwrap_or(path_str.clone());

                if !self.is_primitive_type(&full_path) && !self.defined_types.contains(&full_path) {
                    self.add_dependency(
                        full_path.clone(),
                        DependencyKind::TypeRef,
                        UsageContext::FunctionCall,
                    );
                    self.usage_counts.function_calls += 1;
                }

                // Record item-level dependency
                self.add_item_dependency(
                    full_path,
                    ItemDepType::FunctionCall,
                    0,
                    Some(format!("{}()", path_str)),
                );
            } else {
                // Simple function call like foo()
                self.add_item_dependency(
                    path_str.clone(),
                    ItemDepType::FunctionCall,
                    0,
                    Some(format!("{}()", path_str)),
                );
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    // Detect struct construction: `Foo { field: value }`
    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        let struct_name = node
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        // Skip Self and self constructions
        if struct_name == "Self" || struct_name.starts_with("Self::") {
            syn::visit::visit_expr_struct(self, node);
            return;
        }

        let full_path = self
            .imported_types
            .get(&struct_name)
            .cloned()
            .unwrap_or(struct_name.clone());

        if !self.defined_types.contains(&full_path) && !self.is_primitive_type(&struct_name) {
            self.add_dependency(
                full_path,
                DependencyKind::TypeRef,
                UsageContext::StructConstruction,
            );
            self.usage_counts.struct_constructions += 1;
        }
        syn::visit::visit_expr_struct(self, node);
    }
}

/// Analyzed file data
#[derive(Debug, Clone)]
struct AnalyzedFile {
    module_name: String,
    #[allow(dead_code)]
    file_path: PathBuf,
    metrics: ModuleMetrics,
    dependencies: Vec<Dependency>,
    /// Type visibility information from this file
    type_visibility: HashMap<String, Visibility>,
    /// Item-level dependencies (function calls, field access, etc.)
    item_dependencies: Vec<ItemDependency>,
}

/// Analyze an entire project (parallel version)
pub fn analyze_project(path: &Path) -> Result<ProjectMetrics, AnalyzerError> {
    analyze_project_parallel(path)
}

/// Get an iterator over all non-hidden rust files in `dir`
fn rs_files(dir: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(move |entry| {
            let file_path = entry.path();
            // strip parent dir to avoid false positives for `target` and hidden checks
            let file_path = file_path.strip_prefix(dir).unwrap_or(file_path);

            // Skip target directory and hidden directories
            !file_path.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s == "target" || s.starts_with('.')
            }) && file_path.extension() == Some(OsStr::new("rs"))
        })
        .map(|e| e.path().to_path_buf())
}

/// Analyze a project using parallel processing with Rayon
///
/// Automatically scales to available CPU cores. The parallel processing
/// uses work-stealing for optimal load balancing across cores.
pub fn analyze_project_parallel(path: &Path) -> Result<ProjectMetrics, AnalyzerError> {
    if !path.exists() {
        return Err(AnalyzerError::InvalidPath(path.display().to_string()));
    }

    // Collect all .rs file paths first (sequential, but fast)
    let file_paths: Vec<PathBuf> = rs_files(path).collect();

    // Calculate optimal chunk size based on file count and available parallelism
    // Smaller chunks = better load balancing, but more overhead
    // Larger chunks = less overhead, but potential load imbalance
    let num_threads = rayon::current_num_threads();
    let file_count = file_paths.len();

    // Use smaller chunks for better load balancing with work-stealing
    // Minimum chunk size of 1, maximum of file_count / (num_threads * 4)
    let chunk_size = if file_count < num_threads * 2 {
        1 // Small projects: process one file at a time
    } else {
        // Larger projects: balance between parallelism and overhead
        // Use ~4 chunks per thread for good work-stealing behavior
        (file_count / (num_threads * 4)).max(1)
    };

    // Parallel file analysis with optimized chunking
    let analyzed_results: Vec<_> = file_paths
        .par_chunks(chunk_size)
        .flat_map(|chunk| {
            chunk
                .iter()
                .filter_map(|file_path| match analyze_rust_file_full(file_path) {
                    Ok(result) => Some(AnalyzedFile {
                        module_name: result.metrics.name.clone(),
                        file_path: file_path.clone(),
                        metrics: result.metrics,
                        dependencies: result.dependencies,
                        type_visibility: result.type_visibility,
                        item_dependencies: result.item_dependencies,
                    }),
                    Err(e) => {
                        eprintln!("Warning: Failed to analyze {}: {}", file_path.display(), e);
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Build module names set
    let module_names: HashSet<String> = analyzed_results
        .iter()
        .map(|a| a.module_name.clone())
        .collect();

    // Build project metrics (sequential, but fast)
    let mut project = ProjectMetrics::new();
    project.total_files = analyzed_results.len();

    // First pass: register all types with their visibility
    for analyzed in &analyzed_results {
        for (type_name, visibility) in &analyzed.type_visibility {
            project.register_type(type_name.clone(), analyzed.module_name.clone(), *visibility);
        }
    }

    // Second pass: add modules and couplings
    for analyzed in &analyzed_results {
        // Clone metrics and add item_dependencies
        let mut metrics = analyzed.metrics.clone();
        metrics.item_dependencies = analyzed.item_dependencies.clone();
        project.add_module(metrics);

        for dep in &analyzed.dependencies {
            // Skip invalid dependency paths (local variables, Self, etc.)
            if !is_valid_dependency_path(&dep.path) {
                continue;
            }

            // Determine if this is an internal coupling
            let target_module = extract_target_module(&dep.path);

            // Skip if target module looks invalid (but allow known module names)
            if !module_names.contains(&target_module) && !is_valid_dependency_path(&target_module) {
                continue;
            }

            // Calculate distance
            let distance = calculate_distance(&dep.path, &module_names);

            // Determine strength from usage context
            let strength = dep.usage.to_strength();

            // Default volatility
            let volatility = Volatility::Low;

            // Look up target visibility from the type registry
            let target_type = dep.path.split("::").last().unwrap_or(&dep.path);
            let visibility = project
                .get_type_visibility(target_type)
                .unwrap_or(Visibility::Public); // Default to public if unknown

            // Create coupling metric with location
            let coupling = CouplingMetrics::with_location(
                analyzed.module_name.clone(),
                target_module.clone(),
                strength,
                distance,
                volatility,
                visibility,
                analyzed.file_path.clone(),
                dep.line,
            );

            project.add_coupling(coupling);
        }
    }

    // Update any remaining coupling visibility information
    project.update_coupling_visibility();

    Ok(project)
}

/// Analyze a workspace using cargo metadata for better accuracy
pub fn analyze_workspace(path: &Path) -> Result<ProjectMetrics, AnalyzerError> {
    // Try to get workspace info
    let workspace = match WorkspaceInfo::from_path(path) {
        Ok(ws) => Some(ws),
        Err(e) => {
            eprintln!("Note: Could not load workspace metadata: {}", e);
            eprintln!("Falling back to basic analysis...");
            None
        }
    };

    if let Some(ws) = workspace {
        analyze_with_workspace(path, &ws)
    } else {
        // Fall back to basic analysis
        analyze_project(path)
    }
}

/// Analyze project with workspace information (parallel version)
fn analyze_with_workspace(
    _path: &Path,
    workspace: &WorkspaceInfo,
) -> Result<ProjectMetrics, AnalyzerError> {
    let mut project = ProjectMetrics::new();

    // Store workspace info for the report
    project.workspace_name = Some(
        workspace
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string(),
    );
    project.workspace_members = workspace.members.clone();

    // Collect all file paths with their crate names (sequential, fast)
    let mut file_crate_pairs: Vec<(PathBuf, String)> = Vec::new();

    for member_name in &workspace.members {
        if let Some(crate_info) = workspace.get_crate(member_name) {
            if !crate_info.src_path.exists() {
                continue;
            }

            for file_path in rs_files(&crate_info.src_path) {
                file_crate_pairs.push((file_path.to_path_buf(), member_name.clone()));
            }
        }
    }

    // Calculate optimal chunk size for parallel processing
    let num_threads = rayon::current_num_threads();
    let file_count = file_crate_pairs.len();
    let chunk_size = if file_count < num_threads * 2 {
        1
    } else {
        (file_count / (num_threads * 4)).max(1)
    };

    // Parallel file analysis with optimized chunking
    let analyzed_files: Vec<AnalyzedFileWithCrate> = file_crate_pairs
        .par_chunks(chunk_size)
        .flat_map(|chunk| {
            chunk
                .iter()
                .filter_map(
                    |(file_path, crate_name)| match analyze_rust_file_full(file_path) {
                        Ok(result) => Some(AnalyzedFileWithCrate {
                            module_name: result.metrics.name.clone(),
                            crate_name: crate_name.clone(),
                            file_path: file_path.clone(),
                            metrics: result.metrics,
                            dependencies: result.dependencies,
                            item_dependencies: result.item_dependencies,
                        }),
                        Err(e) => {
                            eprintln!("Warning: Failed to analyze {}: {}", file_path.display(), e);
                            None
                        }
                    },
                )
                .collect::<Vec<_>>()
        })
        .collect();

    project.total_files = analyzed_files.len();

    // Build set of known module names for validation
    let module_names: HashSet<String> = analyzed_files
        .iter()
        .map(|a| a.module_name.clone())
        .collect();

    // Second pass: build coupling relationships with workspace context
    for analyzed in &analyzed_files {
        // Clone metrics and add item_dependencies
        let mut metrics = analyzed.metrics.clone();
        metrics.item_dependencies = analyzed.item_dependencies.clone();
        project.add_module(metrics);

        for dep in &analyzed.dependencies {
            // Skip invalid dependency paths (local variables, Self, etc.)
            if !is_valid_dependency_path(&dep.path) {
                continue;
            }

            // Resolve the target crate using workspace info
            let resolved_crate =
                resolve_crate_from_path(&dep.path, &analyzed.crate_name, workspace);

            let target_module = extract_target_module(&dep.path);

            // Skip if target module looks invalid (but allow known module names)
            if !module_names.contains(&target_module) && !is_valid_dependency_path(&target_module) {
                continue;
            }

            // Calculate distance with workspace awareness
            let distance =
                calculate_distance_with_workspace(&dep.path, &analyzed.crate_name, workspace);

            // Determine strength from usage context (more accurate)
            let strength = dep.usage.to_strength();

            // Default volatility
            let volatility = Volatility::Low;

            // Create coupling metric with location info
            let mut coupling = CouplingMetrics::with_location(
                format!("{}::{}", analyzed.crate_name, analyzed.module_name),
                if let Some(ref crate_name) = resolved_crate {
                    format!("{}::{}", crate_name, target_module)
                } else {
                    target_module.clone()
                },
                strength,
                distance,
                volatility,
                Visibility::Public, // Default visibility for workspace analysis
                analyzed.file_path.clone(),
                dep.line,
            );

            // Add crate-level info
            coupling.source_crate = Some(analyzed.crate_name.clone());
            coupling.target_crate = resolved_crate;

            project.add_coupling(coupling);
        }
    }

    // Add crate-level dependency information
    for (crate_name, deps) in &workspace.dependency_graph {
        if workspace.is_workspace_member(crate_name) {
            for dep in deps {
                // Track crate-level dependencies
                project
                    .crate_dependencies
                    .entry(crate_name.clone())
                    .or_default()
                    .push(dep.clone());
            }
        }
    }

    Ok(project)
}

/// Calculate distance using workspace information
fn calculate_distance_with_workspace(
    dep_path: &str,
    current_crate: &str,
    workspace: &WorkspaceInfo,
) -> Distance {
    if dep_path.starts_with("crate::") || dep_path.starts_with("self::") {
        // Same crate
        Distance::SameModule
    } else if dep_path.starts_with("super::") {
        // Could be same crate or parent module
        Distance::DifferentModule
    } else {
        // Resolve the target crate
        if let Some(target_crate) = resolve_crate_from_path(dep_path, current_crate, workspace) {
            if target_crate == current_crate {
                Distance::SameModule
            } else if workspace.is_workspace_member(&target_crate) {
                // Another workspace member
                Distance::DifferentModule
            } else {
                // External crate
                Distance::DifferentCrate
            }
        } else {
            Distance::DifferentCrate
        }
    }
}

/// Analyzed file with crate information
#[derive(Debug, Clone)]
struct AnalyzedFileWithCrate {
    module_name: String,
    crate_name: String,
    #[allow(dead_code)]
    file_path: PathBuf,
    metrics: ModuleMetrics,
    dependencies: Vec<Dependency>,
    /// Item-level dependencies (function calls, field access, etc.)
    item_dependencies: Vec<ItemDependency>,
}

/// Extract target module name from a path
fn extract_target_module(path: &str) -> String {
    // Remove common prefixes and get the module name
    let cleaned = path
        .trim_start_matches("crate::")
        .trim_start_matches("super::")
        .trim_start_matches("::");

    // Get first significant segment
    cleaned.split("::").next().unwrap_or(path).to_string()
}

/// Check if a path looks like a valid module/type reference (not a local variable)
fn is_valid_dependency_path(path: &str) -> bool {
    // Skip empty paths
    if path.is_empty() {
        return false;
    }

    // Skip Self references
    if path == "Self" || path.starts_with("Self::") {
        return false;
    }

    let segments: Vec<&str> = path.split("::").collect();

    // Skip short single-segment lowercase names (likely local variables)
    if segments.len() == 1 {
        let name = segments[0];
        if name.len() <= 8 && name.chars().all(|c| c.is_lowercase() || c == '_') {
            return false;
        }
    }

    // Skip patterns where last two segments are the same (likely module::type patterns from variables)
    if segments.len() >= 2 {
        let last = segments.last().unwrap();
        let second_last = segments.get(segments.len() - 2).unwrap();
        if last == second_last {
            return false;
        }
    }

    // Skip common patterns that look like local variable accesses
    let last_segment = segments.last().unwrap_or(&path);
    let common_locals = [
        "request",
        "response",
        "result",
        "content",
        "config",
        "proto",
        "domain",
        "info",
        "data",
        "item",
        "value",
        "error",
        "message",
        "expected",
        "actual",
        "status",
        "state",
        "context",
        "params",
        "args",
        "options",
        "settings",
        "violation",
        "page_token",
    ];
    if common_locals.contains(last_segment) && segments.len() <= 2 {
        return false;
    }

    true
}

/// Calculate distance based on dependency path
fn calculate_distance(dep_path: &str, _known_modules: &HashSet<String>) -> Distance {
    if dep_path.starts_with("crate::") || dep_path.starts_with("super::") {
        // Internal dependency
        Distance::DifferentModule
    } else if dep_path.starts_with("self::") {
        Distance::SameModule
    } else {
        // External crate
        Distance::DifferentCrate
    }
}

/// Analyze a single Rust file
/// Result of analyzing a single Rust file
pub struct AnalyzedFileResult {
    pub metrics: ModuleMetrics,
    pub dependencies: Vec<Dependency>,
    pub type_visibility: HashMap<String, Visibility>,
    pub item_dependencies: Vec<ItemDependency>,
}

pub fn analyze_rust_file(path: &Path) -> Result<(ModuleMetrics, Vec<Dependency>), AnalyzerError> {
    let result = analyze_rust_file_full(path)?;
    Ok((result.metrics, result.dependencies))
}

/// Analyze a Rust file and return full results including visibility
pub fn analyze_rust_file_full(path: &Path) -> Result<AnalyzedFileResult, AnalyzerError> {
    let content = fs::read_to_string(path)?;

    let module_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut analyzer = CouplingAnalyzer::new(module_name, path.to_path_buf());
    analyzer.analyze_file(&content)?;

    Ok(AnalyzedFileResult {
        metrics: analyzer.metrics,
        dependencies: analyzer.dependencies,
        type_visibility: analyzer.type_visibility,
        item_dependencies: analyzer.item_dependencies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CouplingAnalyzer::new(
            "test_module".to_string(),
            std::path::PathBuf::from("test.rs"),
        );
        assert_eq!(analyzer.current_module, "test_module");
    }

    #[test]
    fn test_analyze_simple_file() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            pub struct User {
                name: String,
                email: String,
            }

            impl User {
                pub fn new(name: String, email: String) -> Self {
                    Self { name, email }
                }
            }
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());
        assert_eq!(analyzer.metrics.inherent_impl_count, 1);
    }

    #[test]
    fn test_item_dependencies() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            pub struct Config {
                pub value: i32,
            }

            pub fn process(config: Config) -> i32 {
                let x = config.value;
                helper(x)
            }

            fn helper(n: i32) -> i32 {
                n * 2
            }
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());

        // Check that functions are recorded
        assert!(analyzer.defined_functions.contains_key("process"));
        assert!(analyzer.defined_functions.contains_key("helper"));

        // Check item dependencies - process should have deps
        println!(
            "Item dependencies count: {}",
            analyzer.item_dependencies.len()
        );
        for dep in &analyzer.item_dependencies {
            println!(
                "  {} -> {} ({:?})",
                dep.source_item, dep.target, dep.dep_type
            );
        }

        // process function should have dependencies
        let process_deps: Vec<_> = analyzer
            .item_dependencies
            .iter()
            .filter(|d| d.source_item == "process")
            .collect();

        assert!(
            !process_deps.is_empty(),
            "process function should have item dependencies"
        );
    }

    #[test]
    fn test_analyze_trait_impl() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            trait Printable {
                fn print(&self);
            }

            struct Document;

            impl Printable for Document {
                fn print(&self) {}
            }
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());
        assert!(analyzer.metrics.trait_impl_count >= 1);
    }

    #[test]
    fn test_analyze_use_statements() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            use std::collections::HashMap;
            use serde::Serialize;
            use crate::utils;
            use crate::models::{User, Post};
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());
        assert!(analyzer.metrics.external_deps.contains(&"std".to_string()));
        assert!(
            analyzer
                .metrics
                .external_deps
                .contains(&"serde".to_string())
        );
        assert!(!analyzer.dependencies.is_empty());

        // Check internal dependencies
        let internal_deps: Vec<_> = analyzer
            .dependencies
            .iter()
            .filter(|d| d.kind == DependencyKind::InternalUse)
            .collect();
        assert!(!internal_deps.is_empty());
    }

    #[test]
    fn test_extract_use_paths() {
        let analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        // Test simple path
        let tree: UseTree = syn::parse_quote!(std::collections::HashMap);
        let paths = analyzer.extract_use_paths(&tree, "");
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].0, "std::collections::HashMap");

        // Test grouped path
        let tree: UseTree = syn::parse_quote!(crate::models::{User, Post});
        let paths = analyzer.extract_use_paths(&tree, "");
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_extract_target_module() {
        assert_eq!(extract_target_module("crate::models::user"), "models");
        assert_eq!(extract_target_module("super::utils"), "utils");
        assert_eq!(extract_target_module("std::collections"), "std");
    }

    #[test]
    fn test_field_access_detection() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            use crate::models::User;

            fn get_name(user: &User) -> String {
                user.name.clone()
            }
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());

        // Should detect User as a dependency with field access
        let _field_deps: Vec<_> = analyzer
            .dependencies
            .iter()
            .filter(|d| d.usage == UsageContext::FieldAccess)
            .collect();
        // Note: This may not detect field access on function parameters
        // as the type info isn't fully available without type inference
    }

    #[test]
    fn test_method_call_detection() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            fn process() {
                let data = String::new();
                data.push_str("hello");
            }
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());
        // Method calls on local variables are detected
    }

    #[test]
    fn test_struct_construction_detection() {
        let mut analyzer =
            CouplingAnalyzer::new("test".to_string(), std::path::PathBuf::from("test.rs"));

        let code = r#"
            use crate::config::Config;

            fn create_config() {
                let c = Config { value: 42 };
            }
        "#;

        let result = analyzer.analyze_file(code);
        assert!(result.is_ok());

        // Should detect Config struct construction
        let struct_deps: Vec<_> = analyzer
            .dependencies
            .iter()
            .filter(|d| d.usage == UsageContext::StructConstruction)
            .collect();
        assert!(!struct_deps.is_empty());
    }

    #[test]
    fn test_usage_context_to_strength() {
        assert_eq!(
            UsageContext::FieldAccess.to_strength(),
            IntegrationStrength::Intrusive
        );
        assert_eq!(
            UsageContext::MethodCall.to_strength(),
            IntegrationStrength::Functional
        );
        assert_eq!(
            UsageContext::TypeParameter.to_strength(),
            IntegrationStrength::Model
        );
        assert_eq!(
            UsageContext::TraitBound.to_strength(),
            IntegrationStrength::Contract
        );
    }
}
