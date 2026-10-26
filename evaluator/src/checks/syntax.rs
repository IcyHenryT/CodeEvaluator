use std::collections::{HashMap, HashSet};

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, BinOp, Expr, Item, ItemFn, Meta, Pat, Stmt, Token, Visibility};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyntaxAnalysis {
    pub for_loop_count: usize,
    pub while_loop_count: usize,
    pub loop_count: usize,
    pub unsafe_count: usize,
    pub clone_count: usize,
    pub has_recursion: bool,
    pub variable_count: usize,
    pub mutable_binding_count: usize,
    pub assignment_count: usize,
    pub if_expression_count: usize,
}

pub fn analyze_syntax(code: &str) -> Result<SyntaxAnalysis, syn::Error> {
    let file = syn::parse_file(code)?;
    Ok(analyze_file(&file))
}

pub(crate) fn analyze_file(file: &syn::File) -> SyntaxAnalysis {
    let mut visitor = WholeFileVisitor::default();
    visitor.visit_file(file);
    visitor.analysis
}

pub fn analyze_requirement_syntax(code: &str) -> Result<SyntaxAnalysis, syn::Error> {
    let file = syn::parse_file(code)?;
    Ok(analyze_requirement_file(&file, None).0)
}

pub(crate) fn analyze_requirement_file(
    file: &syn::File,
    entry_points: Option<&[&str]>,
) -> (SyntaxAnalysis, Vec<String>) {
    let mut missing_entry_points: Vec<String> = entry_points
        .unwrap_or_default()
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    if !attributes_enabled(&file.attrs) {
        return (SyntaxAnalysis::default(), missing_entry_points);
    }

    let functions: HashMap<_, _> = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(function) if attributes_enabled(&function.attrs) => {
                Some((function.sig.ident.to_string(), function))
            }
            _ => None,
        })
        .collect();
    let mut visitor = ReachabilityVisitor {
        analysis: SyntaxAnalysis::default(),
        current_function: None,
        scopes: vec![Scope {
            functions,
            bindings: HashSet::new(),
        }],
        visited_functions: HashSet::new(),
    };

    for item in &file.items {
        if let Item::Fn(function) = item
            && matches!(function.vis, Visibility::Public(_))
            && attributes_enabled(&function.attrs)
        {
            let name = function.sig.ident.to_string();
            missing_entry_points.retain(|entry_point| entry_point != &name);
            if entry_points.is_none_or(|names| names.contains(&name.as_str())) {
                visitor.visit_function(function, 0);
            }
        }
    }
    (visitor.analysis, missing_entry_points)
}

impl SyntaxAnalysis {
    fn record_expression(&mut self, node: &Expr, current_function: Option<&str>) {
        match node {
            Expr::ForLoop(node) => {
                self.for_loop_count += 1;
                self.mutable_binding_count += pattern_bindings(&node.pat).mutable_count;
            }
            Expr::While(_) => self.while_loop_count += 1,
            Expr::Loop(_) => self.loop_count += 1,
            Expr::Unsafe(_) => self.unsafe_count += 1,
            Expr::If(_) => self.if_expression_count += 1,
            Expr::Assign(_) => self.assignment_count += 1,
            Expr::Binary(node)
                if matches!(
                    node.op,
                    BinOp::AddAssign(_)
                        | BinOp::SubAssign(_)
                        | BinOp::MulAssign(_)
                        | BinOp::DivAssign(_)
                        | BinOp::RemAssign(_)
                        | BinOp::BitXorAssign(_)
                        | BinOp::BitAndAssign(_)
                        | BinOp::BitOrAssign(_)
                        | BinOp::ShlAssign(_)
                        | BinOp::ShrAssign(_)
                ) =>
            {
                self.assignment_count += 1;
            }
            Expr::Let(node) => {
                self.mutable_binding_count += pattern_bindings(&node.pat).mutable_count;
            }
            Expr::Call(node) => {
                if let (Some(function), Expr::Path(path)) = (current_function, &*node.func)
                    && path
                        .path
                        .segments
                        .last()
                        .is_some_and(|segment| segment.ident == function)
                {
                    self.has_recursion = true;
                }
            }
            Expr::MethodCall(node) if node.method == "clone" => self.clone_count += 1,
            _ => {}
        }
    }

    fn record_local(&mut self, local: &syn::Local) {
        self.variable_count += 1;
        self.mutable_binding_count += pattern_bindings(&local.pat).mutable_count;
    }
}

#[derive(Default)]
struct PatternBindings {
    names: HashSet<String>,
    mutable_count: usize,
}

impl<'ast> Visit<'ast> for PatternBindings {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.names.insert(node.ident.to_string());
        self.mutable_count += node.mutability.is_some() as usize;
        visit::visit_pat_ident(self, node);
    }
}

fn pattern_bindings(pattern: &Pat) -> PatternBindings {
    let mut bindings = PatternBindings::default();
    bindings.visit_pat(pattern);
    bindings
}

#[derive(Default)]
struct WholeFileVisitor {
    analysis: SyntaxAnalysis,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for WholeFileVisitor {
    fn visit_expr(&mut self, node: &'ast Expr) {
        self.analysis
            .record_expression(node, self.current_function.as_deref());
        visit::visit_expr(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        self.analysis.record_local(node);
        visit::visit_local(self, node);
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        self.analysis.mutable_binding_count += pattern_bindings(&node.pat).mutable_count;
        visit::visit_arm(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.analysis.unsafe_count += node.sig.unsafety.is_some() as usize;
        let previous_function = self.current_function.replace(node.sig.ident.to_string());
        visit::visit_item_fn(self, node);
        self.current_function = previous_function;
    }
}

#[derive(Clone, Default)]
struct Scope<'ast> {
    functions: HashMap<String, &'ast ItemFn>,
    bindings: HashSet<String>,
}

struct ReachabilityVisitor<'ast> {
    analysis: SyntaxAnalysis,
    current_function: Option<String>,
    scopes: Vec<Scope<'ast>>,
    visited_functions: HashSet<*const ItemFn>,
}

impl<'ast> ReachabilityVisitor<'ast> {
    fn visit_function(&mut self, function: &'ast ItemFn, declaration_scope: usize) {
        if !self.visited_functions.insert(function as *const ItemFn) {
            return;
        }

        let caller_scopes = std::mem::take(&mut self.scopes);
        self.scopes = caller_scopes[..=declaration_scope].to_vec();
        let mut parameter_scope = Scope::default();
        for input in &function.sig.inputs {
            if let syn::FnArg::Typed(argument) = input {
                parameter_scope
                    .bindings
                    .extend(pattern_bindings(&argument.pat).names);
            }
        }
        self.scopes.push(parameter_scope);
        self.analysis.unsafe_count += function.sig.unsafety.is_some() as usize;
        let previous_function = self
            .current_function
            .replace(function.sig.ident.to_string());
        self.visit_block(&function.block);
        self.current_function = previous_function;
        self.scopes = caller_scopes;
    }

    fn push_bindings(&mut self, names: HashSet<String>) {
        self.scopes.push(Scope {
            bindings: names,
            ..Scope::default()
        });
    }

    fn visit_invoked_closure(&mut self, node: &'ast syn::ExprClosure) {
        if node.asyncness.is_some() {
            return;
        }
        let names = node
            .inputs
            .iter()
            .flat_map(|pattern| pattern_bindings(pattern).names)
            .collect();
        self.push_bindings(names);
        self.visit_expr(&node.body);
        self.scopes.pop();
    }

    fn resolve_call(&self, expression: &Expr) -> Option<(&'ast ItemFn, usize)> {
        let Expr::Path(path) = direct_call_target(expression)? else {
            return None;
        };
        if path.qself.is_some() || path.path.leading_colon.is_some() {
            return None;
        }
        let segments = &path.path.segments;
        let name = segments.last()?.ident.to_string();
        if segments.len() == 2 && (segments[0].ident == "crate" || segments[0].ident == "self") {
            return self.scopes[0].functions.get(&name).map(|node| (*node, 0));
        }
        if segments.len() != 1 {
            return None;
        }
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if scope.bindings.contains(&name) {
                return None;
            }
            if let Some(function) = scope.functions.get(&name) {
                return Some((*function, index));
            }
        }
        None
    }
}

impl<'ast> Visit<'ast> for ReachabilityVisitor<'ast> {
    fn visit_item(&mut self, _node: &'ast Item) {}

    fn visit_block(&mut self, node: &'ast syn::Block) {
        let mut scope = Scope::default();
        for statement in &node.stmts {
            if let Stmt::Item(Item::Fn(function)) = statement
                && attributes_enabled(&function.attrs)
            {
                scope
                    .functions
                    .insert(function.sig.ident.to_string(), function);
            }
        }
        self.scopes.push(scope);
        visit::visit_block(self, node);
        self.scopes.pop();
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        if !expression_enabled(node) {
            return;
        }
        self.analysis
            .record_expression(node, self.current_function.as_deref());
        if let Expr::Call(call) = node {
            if let Some((function, declaration_scope)) = self.resolve_call(&call.func) {
                self.visit_function(function, declaration_scope);
            } else if let Some(Expr::Closure(closure)) = direct_call_target(&call.func) {
                self.visit_invoked_closure(closure);
            }
        }
        visit::visit_expr(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if !attributes_enabled(&node.attrs) {
            return;
        }
        self.analysis.record_local(node);
        visit::visit_local(self, node);
        self.scopes
            .last_mut()
            .unwrap()
            .bindings
            .extend(pattern_bindings(&node.pat).names);
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        if !attributes_enabled(&node.attrs) {
            return;
        }
        let bindings = pattern_bindings(&node.pat);
        self.analysis.mutable_binding_count += bindings.mutable_count;
        self.push_bindings(bindings.names);
        visit::visit_arm(self, node);
        self.scopes.pop();
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.visit_expr(&node.expr);
        self.visit_pat(&node.pat);
        self.push_bindings(pattern_bindings(&node.pat).names);
        self.visit_block(&node.body);
        self.scopes.pop();
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.push_bindings(HashSet::new());
        self.visit_expr(&node.cond);
        self.visit_block(&node.then_branch);
        self.scopes.pop();
        if let Some((_, branch)) = &node.else_branch {
            self.visit_expr(branch);
        }
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.push_bindings(HashSet::new());
        self.visit_expr(&node.cond);
        self.visit_block(&node.body);
        self.scopes.pop();
    }

    fn visit_expr_let(&mut self, node: &'ast syn::ExprLet) {
        visit::visit_expr_let(self, node);
        self.scopes
            .last_mut()
            .unwrap()
            .bindings
            .extend(pattern_bindings(&node.pat).names);
    }

    fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}
}

fn direct_call_target(mut expression: &Expr) -> Option<&Expr> {
    loop {
        if !expression_enabled(expression) {
            return None;
        }
        expression = match expression {
            Expr::Paren(node) => &node.expr,
            Expr::Group(node) => &node.expr,
            _ => return Some(expression),
        };
    }
}

fn expression_enabled(expression: &Expr) -> bool {
    macro_rules! check_attributes {
        ($($variant:ident),* $(,)?) => {
            match expression {
                $(Expr::$variant(node) => attributes_enabled(&node.attrs),)*
                _ => true,
            }
        };
    }
    check_attributes!(
        Array, Assign, Async, Await, Binary, Block, Break, Call, Cast, Closure, Const, Continue,
        Field, ForLoop, Group, If, Index, Infer, Let, Lit, Loop, Macro, Match, MethodCall, Paren,
        Path, Range, RawAddr, Reference, Repeat, Return, Struct, Try, TryBlock, Tuple, Unary,
        Unsafe, While, Yield,
    )
}

fn attributes_enabled(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .all(|attribute| meta_enabled(&attribute.meta))
}

fn meta_enabled(meta: &Meta) -> bool {
    if meta.path().is_ident("test") {
        return false;
    }
    let Meta::List(list) = meta else {
        return true;
    };
    if list.path.is_ident("cfg") {
        return list
            .parse_args::<Meta>()
            .ok()
            .and_then(|condition| cfg_value(&condition))
            == Some(true);
    }
    if list.path.is_ident("cfg_attr") {
        let Ok(arguments) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            return false;
        };
        let mut arguments = arguments.iter();
        let Some(condition) = arguments.next() else {
            return false;
        };
        return cfg_value(condition) == Some(false) || arguments.all(meta_enabled);
    }
    true
}

fn cfg_value(meta: &Meta) -> Option<bool> {
    match meta {
        Meta::Path(path) if path.is_ident("test") => Some(false),
        Meta::List(list) => {
            let arguments = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ok()?;
            if list.path.is_ident("not") && arguments.len() == 1 {
                return cfg_value(&arguments[0]).map(|value| !value);
            }
            let is_all = list.path.is_ident("all");
            if !is_all && !list.path.is_ident("any") {
                return None;
            }
            let mut unknown = false;
            for argument in &arguments {
                match cfg_value(argument) {
                    Some(value) if value != is_all => return Some(value),
                    None => unknown = true,
                    _ => {}
                }
            }
            if unknown { None } else { Some(is_all) }
        }
        _ => None,
    }
}
