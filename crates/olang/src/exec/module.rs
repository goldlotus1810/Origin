//! # module — Module system for Olang
//!
//! Provides ModuleLoader, ModuleCache, and dependency graph with circular
//! dependency detection.
//!
//! ## Module resolution
//!
//! ```text
//! use silk.graph;                → load "silk/graph.ol"
//! use silk.graph.{SilkGraph};   → load "silk/graph.ol", import SilkGraph
//! mod silk.graph;                → declare current file as silk.graph module
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ModuleLoader: path resolution + parse + compile + cache
//! ModuleCache:  compiled modules (OlangProgram + exported symbols)
//! DepGraph:     tracks import edges, detects cycles
//! ```

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ir::OlangProgram;
use crate::syntax::Stmt;

// ─────────────────────────────────────────────────────────────────────────────
// Symbol — exported item from a module
// ─────────────────────────────────────────────────────────────────────────────

/// Visibility of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Accessible from any module
    Public,
    /// Only accessible within the defining module
    Private,
}

/// An exported symbol from a module.
#[derive(Debug, Clone)]
pub struct ModuleSymbol {
    /// Symbol name (function, type, constant)
    pub name: String,
    /// Visibility
    pub vis: Visibility,
    /// Kind of symbol
    pub kind: SymbolKind,
}

/// Kind of exported symbol.
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Function (param count)
    Function(usize),
    /// Struct type (field names)
    Struct(Vec<String>),
    /// Enum type (variant names)
    Enum(Vec<String>),
    /// Trait (method names)
    Trait(Vec<String>),
    /// Constant value
    Constant,
}

// ─────────────────────────────────────────────────────────────────────────────
// CompiledModule — a fully resolved module
// ─────────────────────────────────────────────────────────────────────────────

/// A compiled module with its exports.
#[derive(Debug, Clone)]
pub struct CompiledModule {
    /// Dot-separated module path (e.g. "silk.graph")
    pub path: String,
    /// Compiled IR program
    pub program: OlangProgram,
    /// Exported symbols (pub items)
    pub exports: Vec<ModuleSymbol>,
    /// Modules this module depends on
    pub dependencies: Vec<String>,
}

impl CompiledModule {
    /// Create a new compiled module.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.into(),
            program: OlangProgram::new(path),
            exports: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Check if a symbol is exported (public).
    pub fn has_export(&self, name: &str) -> bool {
        self.exports.iter().any(|s| s.name == name && s.vis == Visibility::Public)
    }

    /// Get an exported symbol by name.
    pub fn get_export(&self, name: &str) -> Option<&ModuleSymbol> {
        self.exports.iter().find(|s| s.name == name && s.vis == Visibility::Public)
    }

    /// List all public symbol names.
    pub fn public_symbols(&self) -> Vec<&str> {
        self.exports.iter()
            .filter(|s| s.vis == Visibility::Public)
            .map(|s| s.name.as_str())
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ModuleCache — stores compiled modules
// ─────────────────────────────────────────────────────────────────────────────

/// Cache of compiled modules, keyed by module path.
#[derive(Debug, Default)]
pub struct ModuleCache {
    modules: Vec<CompiledModule>,
}

impl ModuleCache {
    /// Create empty cache.
    pub fn new() -> Self {
        Self { modules: Vec::new() }
    }

    /// Insert a compiled module. Returns false if already cached.
    pub fn insert(&mut self, module: CompiledModule) -> bool {
        if self.contains(&module.path) {
            return false;
        }
        self.modules.push(module);
        true
    }

    /// Check if a module path is cached.
    pub fn contains(&self, path: &str) -> bool {
        self.modules.iter().any(|m| m.path == path)
    }

    /// Get a cached module by path.
    pub fn get(&self, path: &str) -> Option<&CompiledModule> {
        self.modules.iter().find(|m| m.path == path)
    }

    /// Number of cached modules.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// List all cached module paths.
    pub fn paths(&self) -> Vec<&str> {
        self.modules.iter().map(|m| m.path.as_str()).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DepGraph — dependency graph with cycle detection
// ─────────────────────────────────────────────────────────────────────────────

/// Edge in the dependency graph: (from_module, to_module).
#[derive(Debug, Clone, PartialEq)]
struct DepEdge {
    from: String,
    to: String,
}

/// Dependency graph for modules.
/// Tracks import relationships and detects circular dependencies.
#[derive(Debug, Default)]
pub struct DepGraph {
    edges: Vec<DepEdge>,
}

/// Error when a circular dependency is detected.
#[derive(Debug, Clone, PartialEq)]
pub struct CyclicDependencyError {
    /// The cycle path: ["a", "b", "c", "a"]
    pub cycle: Vec<String>,
}

impl core::fmt::Display for CyclicDependencyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Circular dependency detected: {}", self.cycle.join(" → "))
    }
}

impl DepGraph {
    /// Create empty dependency graph.
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Add a dependency edge: `from` depends on `to`.
    /// Returns Err if this creates a cycle.
    pub fn add_dependency(&mut self, from: &str, to: &str) -> Result<(), CyclicDependencyError> {
        let edge = DepEdge { from: from.into(), to: to.into() };
        if self.edges.contains(&edge) {
            return Ok(()); // already recorded
        }
        // Temporarily add edge
        self.edges.push(edge);
        // Check for cycles
        if let Some(cycle) = self.detect_cycle() {
            self.edges.pop(); // remove the offending edge
            return Err(CyclicDependencyError { cycle });
        }
        Ok(())
    }

    /// Detect any cycle in the graph using DFS.
    fn detect_cycle(&self) -> Option<Vec<String>> {
        // Collect all unique nodes
        let mut nodes = Vec::new();
        for e in &self.edges {
            if !nodes.contains(&e.from) { nodes.push(e.from.clone()); }
            if !nodes.contains(&e.to) { nodes.push(e.to.clone()); }
        }

        for start in &nodes {
            let mut visited = Vec::new();
            let mut stack = Vec::new();
            if self.dfs_cycle(start, &mut visited, &mut stack) {
                stack.push(start.clone());
                return Some(stack);
            }
        }
        None
    }

    /// DFS from `node`, tracking path in `stack`. Returns true if cycle found.
    fn dfs_cycle(&self, node: &str, visited: &mut Vec<String>, stack: &mut Vec<String>) -> bool {
        if stack.contains(&node.to_string()) {
            return true; // back-edge → cycle
        }
        if visited.contains(&node.to_string()) {
            return false; // already fully explored
        }
        stack.push(node.to_string());
        // Visit all neighbors
        let neighbors: Vec<String> = self.edges.iter()
            .filter(|e| e.from == node)
            .map(|e| e.to.clone())
            .collect();
        for neighbor in &neighbors {
            if self.dfs_cycle(neighbor, visited, stack) {
                return true;
            }
        }
        stack.pop();
        visited.push(node.to_string());
        false
    }

    /// Get all direct dependencies of a module.
    pub fn dependencies_of(&self, module: &str) -> Vec<&str> {
        self.edges.iter()
            .filter(|e| e.from == module)
            .map(|e| e.to.as_str())
            .collect()
    }

    /// Get all modules that depend on the given module.
    pub fn dependents_of(&self, module: &str) -> Vec<&str> {
        self.edges.iter()
            .filter(|e| e.to == module)
            .map(|e| e.from.as_str())
            .collect()
    }

    /// Topological sort of all modules. Returns None if cycle exists.
    pub fn topological_sort(&self) -> Option<Vec<String>> {
        let mut nodes = Vec::new();
        for e in &self.edges {
            if !nodes.contains(&e.from) { nodes.push(e.from.clone()); }
            if !nodes.contains(&e.to) { nodes.push(e.to.clone()); }
        }

        // Edge from→to means "from depends on to", so in topological order,
        // "to" must come before "from". We reverse: in-degree = edges pointing TO node.
        // A node with no dependencies (not appearing as `from` in any edge, or
        // only appearing as `to`) has in_degree = 0 in the reversed graph.
        // In-degree for node N = count of edges where N is `from` (N depends on others).
        // Wait — standard: edge from→to means from depends on to.
        // In reversed DAG for topo sort: edge to→from.
        // In-degree in reversed DAG for node N = edges where N appears as `from`.
        let mut in_degree: Vec<(String, usize)> = nodes.iter()
            .map(|n| {
                // Count how many things N depends on = edges where from == N
                let deg = self.edges.iter().filter(|e| e.from == *n).count();
                (n.clone(), deg)
            })
            .collect();

        let mut result = Vec::new();
        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, d)| *d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        let mut queue_front = 0usize;

        while queue_front < queue.len() {
            let node = queue[queue_front].clone();
            queue_front += 1;
            result.push(node.clone());
            // In reversed DAG: node is a dependency. Find all modules that depend on node.
            let dependents: Vec<String> = self.edges.iter()
                .filter(|e| e.to == node)
                .map(|e| e.from.clone())
                .collect();
            for neighbor in &dependents {
                if let Some(entry) = in_degree.iter_mut().find(|(n, _)| n == neighbor) {
                    entry.1 = entry.1.saturating_sub(1);
                    if entry.1 == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        if result.len() == nodes.len() {
            Some(result)
        } else {
            None // cycle exists
        }
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ModuleLoader — orchestrates module loading
// ─────────────────────────────────────────────────────────────────────────────

/// Module resolution error.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleError {
    /// Error description
    pub message: String,
}

impl ModuleError {
    /// Create a new module error.
    pub fn new(msg: &str) -> Self {
        Self { message: msg.into() }
    }
}

impl core::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ModuleError: {}", self.message)
    }
}

/// Module loader: resolves, parses, compiles, and caches modules.
///
/// ## Resolution rules
///
/// Module path `silk.graph` resolves to file `silk/graph.ol`.
/// The loader searches root directories in order.
#[derive(Debug)]
pub struct ModuleLoader {
    /// Module cache
    pub cache: ModuleCache,
    /// Dependency graph
    pub deps: DepGraph,
    /// Root directories to search for modules
    pub roots: Vec<String>,
    /// Currently loading modules (for cycle detection during loading)
    loading_stack: Vec<String>,
}

impl ModuleLoader {
    /// Create a new module loader with given root directories.
    pub fn new(roots: Vec<String>) -> Self {
        Self {
            cache: ModuleCache::new(),
            deps: DepGraph::new(),
            roots,
            loading_stack: Vec::new(),
        }
    }

    /// Convert module path to file path.
    /// `silk.graph` → `silk/graph.ol`
    pub fn resolve_path(module_path: &str) -> String {
        let file_path = module_path.replace('.', "/");
        alloc::format!("{}.ol", file_path)
    }

    /// Register a module directly (without loading from file).
    /// Used for built-in modules and testing.
    pub fn register(&mut self, module: CompiledModule) -> Result<(), ModuleError> {
        // Record dependencies
        for dep in &module.dependencies {
            self.deps.add_dependency(&module.path, dep)
                .map_err(|e| ModuleError::new(&alloc::format!("{}", e)))?;
        }
        if !self.cache.insert(module) {
            // Already cached — not an error, just skip
        }
        Ok(())
    }

    /// Check if loading a module would create a circular dependency.
    pub fn would_cycle(&self, from: &str, to: &str) -> bool {
        let mut test_deps = DepGraph::new();
        // Copy existing edges
        for e in &self.deps.edges {
            let _ = test_deps.add_dependency(&e.from, &e.to);
        }
        test_deps.add_dependency(from, to).is_err()
    }

    /// Begin loading a module (push onto loading stack).
    /// Returns Err if the module is already being loaded (circular import at load time).
    pub fn begin_loading(&mut self, path: &str) -> Result<(), ModuleError> {
        if self.loading_stack.contains(&path.to_string()) {
            let mut cycle = self.loading_stack.clone();
            cycle.push(path.into());
            return Err(ModuleError::new(&alloc::format!(
                "Circular import detected during loading: {}",
                cycle.join(" → ")
            )));
        }
        self.loading_stack.push(path.into());
        Ok(())
    }

    /// Finish loading a module (pop from loading stack).
    pub fn finish_loading(&mut self, path: &str) {
        if let Some(pos) = self.loading_stack.iter().position(|p| p == path) {
            self.loading_stack.remove(pos);
        }
    }

    /// Resolve a selective import: check that all requested symbols exist and are public.
    pub fn resolve_imports(
        &self,
        module_path: &str,
        imports: &[String],
    ) -> Result<Vec<ModuleSymbol>, ModuleError> {
        let module = self.cache.get(module_path).ok_or_else(|| {
            ModuleError::new(&alloc::format!("Module '{}' not found", module_path))
        })?;

        let mut resolved = Vec::new();
        for name in imports {
            match module.get_export(name) {
                Some(sym) => resolved.push(sym.clone()),
                None => {
                    // Check if it exists but is private
                    let exists_private = module.exports.iter().any(|s| s.name == *name);
                    if exists_private {
                        return Err(ModuleError::new(&alloc::format!(
                            "Symbol '{}' in module '{}' is private",
                            name, module_path
                        )));
                    }
                    return Err(ModuleError::new(&alloc::format!(
                        "Symbol '{}' not found in module '{}'",
                        name, module_path
                    )));
                }
            }
        }
        Ok(resolved)
    }

    /// Get the load order for all modules (topological sort).
    pub fn load_order(&self) -> Option<Vec<String>> {
        self.deps.topological_sort()
    }

    /// Check if a module is cached.
    pub fn is_loaded(&self, path: &str) -> bool {
        self.cache.contains(path)
    }

    /// Load a module from source code: parse → compile → extract exports → cache.
    ///
    /// `requester` is the module requesting the import (for cycle detection).
    /// Returns the compiled module's public symbol names.
    pub fn load_from_source(
        &mut self,
        module_path: &str,
        source: &str,
        requester: Option<&str>,
    ) -> Result<Vec<String>, ModuleError> {
        // Already cached?
        if let Some(cached) = self.cache.get(module_path) {
            return Ok(cached.public_symbols().iter().map(|s| s.to_string()).collect());
        }

        // Circular import check
        self.begin_loading(module_path)?;

        // Record dependency if there's a requester
        if let Some(req) = requester {
            self.deps.add_dependency(req, module_path)
                .map_err(|e| ModuleError::new(&alloc::format!("{}", e)))?;
        }

        // Parse
        let stmts = crate::syntax::parse(source)
            .map_err(|e| ModuleError::new(&alloc::format!("Parse error: {:?}", e)))?;

        // Extract exports from AST
        let exports = extract_exports(&stmts);

        // Compile
        let program = crate::semantic::lower(&stmts);

        // Build CompiledModule
        let mut module = CompiledModule {
            path: module_path.into(),
            program,
            exports,
            dependencies: Vec::new(),
        };

        // Record dependencies from `use` statements in the module
        for stmt in &stmts {
            if let Stmt::Use { module: dep, .. } = stmt {
                module.dependencies.push(dep.clone());
            }
        }

        let pub_names = module.public_symbols().iter().map(|s| s.to_string()).collect();

        self.finish_loading(module_path);
        self.cache.insert(module);

        Ok(pub_names)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// extract_exports — scan AST for pub items
// ─────────────────────────────────────────────────────────────────────────────

/// Extract exported (public) symbols from a parsed AST.
/// Scans for `Stmt::Pub(inner)` wrappers around fn/struct/enum/trait definitions.
pub fn extract_exports(stmts: &[Stmt]) -> Vec<ModuleSymbol> {
    let mut exports = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Pub(inner) => {
                if let Some(sym) = stmt_to_symbol(inner, Visibility::Public) {
                    exports.push(sym);
                }
            }
            // Non-pub top-level items are private
            other => {
                if let Some(sym) = stmt_to_symbol(other, Visibility::Private) {
                    exports.push(sym);
                }
            }
        }
    }
    exports
}

/// Convert a statement to a ModuleSymbol if it's a named definition.
fn stmt_to_symbol(stmt: &Stmt, vis: Visibility) -> Option<ModuleSymbol> {
    match stmt {
        Stmt::FnDef { name, params, .. } => Some(ModuleSymbol {
            name: name.clone(),
            vis,
            kind: SymbolKind::Function(params.len()),
        }),
        Stmt::StructDef { name, fields, .. } => Some(ModuleSymbol {
            name: name.clone(),
            vis,
            kind: SymbolKind::Struct(fields.iter().map(|f| f.name.clone()).collect()),
        }),
        Stmt::EnumDef { name, variants, .. } => Some(ModuleSymbol {
            name: name.clone(),
            vis,
            kind: SymbolKind::Enum(variants.iter().map(|v| v.name.clone()).collect()),
        }),
        Stmt::TraitDef { name, methods, .. } => Some(ModuleSymbol {
            name: name.clone(),
            vis,
            kind: SymbolKind::Trait(methods.iter().map(|m| m.name.clone()).collect()),
        }),
        Stmt::Let { name, .. } => Some(ModuleSymbol {
            name: name.clone(),
            vis,
            kind: SymbolKind::Constant,
        }),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ModuleCache tests ────────────────────────────────────────────────

    #[test]
    fn cache_new_empty() {
        let cache = ModuleCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cache_insert_and_get() {
        let mut cache = ModuleCache::new();
        let mut m = CompiledModule::new("silk.graph");
        m.exports.push(ModuleSymbol {
            name: "SilkGraph".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Struct(alloc::vec!["nodes".into(), "edges".into()]),
        });
        assert!(cache.insert(m));
        assert!(cache.contains("silk.graph"));
        assert!(!cache.contains("silk.walk"));
        assert_eq!(cache.len(), 1);
        let module = cache.get("silk.graph").unwrap();
        assert!(module.has_export("SilkGraph"));
    }

    #[test]
    fn cache_duplicate_insert_returns_false() {
        let mut cache = ModuleCache::new();
        assert!(cache.insert(CompiledModule::new("math")));
        assert!(!cache.insert(CompiledModule::new("math")));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_paths() {
        let mut cache = ModuleCache::new();
        cache.insert(CompiledModule::new("a"));
        cache.insert(CompiledModule::new("b"));
        cache.insert(CompiledModule::new("c"));
        let paths = cache.paths();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"a"));
        assert!(paths.contains(&"b"));
        assert!(paths.contains(&"c"));
    }

    // ── DepGraph tests ──────────────────────────────────────────────────

    #[test]
    fn dep_graph_no_cycle() {
        let mut g = DepGraph::new();
        assert!(g.add_dependency("a", "b").is_ok());
        assert!(g.add_dependency("b", "c").is_ok());
        assert!(g.add_dependency("a", "c").is_ok());
        assert_eq!(g.edge_count(), 3);
    }

    #[test]
    fn dep_graph_direct_cycle() {
        let mut g = DepGraph::new();
        assert!(g.add_dependency("a", "b").is_ok());
        let result = g.add_dependency("b", "a");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.cycle.contains(&"a".to_string()));
        assert!(err.cycle.contains(&"b".to_string()));
    }

    #[test]
    fn dep_graph_indirect_cycle() {
        let mut g = DepGraph::new();
        assert!(g.add_dependency("a", "b").is_ok());
        assert!(g.add_dependency("b", "c").is_ok());
        let result = g.add_dependency("c", "a");
        assert!(result.is_err());
    }

    #[test]
    fn dep_graph_duplicate_edge_ok() {
        let mut g = DepGraph::new();
        assert!(g.add_dependency("a", "b").is_ok());
        assert!(g.add_dependency("a", "b").is_ok()); // duplicate
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn dep_graph_dependencies_of() {
        let mut g = DepGraph::new();
        let _ = g.add_dependency("app", "silk");
        let _ = g.add_dependency("app", "context");
        let _ = g.add_dependency("silk", "olang");
        let deps = g.dependencies_of("app");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"silk"));
        assert!(deps.contains(&"context"));
    }

    #[test]
    fn dep_graph_dependents_of() {
        let mut g = DepGraph::new();
        let _ = g.add_dependency("app", "silk");
        let _ = g.add_dependency("learning", "silk");
        let dependents = g.dependents_of("silk");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"app"));
        assert!(dependents.contains(&"learning"));
    }

    #[test]
    fn dep_graph_topological_sort() {
        let mut g = DepGraph::new();
        let _ = g.add_dependency("app", "silk");
        let _ = g.add_dependency("app", "context");
        let _ = g.add_dependency("silk", "olang");
        let _ = g.add_dependency("context", "olang");
        let order = g.topological_sort().unwrap();
        assert_eq!(order.len(), 4);
        // Verify: all deps come before dependents
        let pos = |name: &str| order.iter().position(|n| n == name).unwrap();
        // app depends on silk and context, so app must come after both
        assert!(pos("app") > pos("silk"), "app should come after silk");
        assert!(pos("app") > pos("context"), "app should come after context");
        // silk depends on olang
        assert!(pos("silk") > pos("olang"), "silk should come after olang");
        // context depends on olang
        assert!(pos("context") > pos("olang"), "context should come after olang");
    }

    #[test]
    fn dep_graph_topological_sort_cycle_returns_none() {
        let mut g = DepGraph::new();
        // Force edges without cycle check to create a cycle
        g.edges.push(DepEdge { from: "a".into(), to: "b".into() });
        g.edges.push(DepEdge { from: "b".into(), to: "a".into() });
        assert!(g.topological_sort().is_none());
    }

    // ── ModuleLoader tests ──────────────────────────────────────────────

    #[test]
    fn loader_resolve_path() {
        assert_eq!(ModuleLoader::resolve_path("silk.graph"), "silk/graph.ol");
        assert_eq!(ModuleLoader::resolve_path("math"), "math.ol");
        assert_eq!(ModuleLoader::resolve_path("agents.learning"), "agents/learning.ol");
    }

    #[test]
    fn loader_register_and_query() {
        let mut loader = ModuleLoader::new(alloc::vec!["stdlib".into()]);
        let mut m = CompiledModule::new("math");
        m.exports.push(ModuleSymbol {
            name: "sin".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Function(1),
        });
        m.exports.push(ModuleSymbol {
            name: "cos".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Function(1),
        });
        m.exports.push(ModuleSymbol {
            name: "_internal".into(),
            vis: Visibility::Private,
            kind: SymbolKind::Function(0),
        });
        assert!(loader.register(m).is_ok());
        assert!(loader.is_loaded("math"));
        assert!(!loader.is_loaded("io"));
    }

    #[test]
    fn loader_resolve_imports_public() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let mut m = CompiledModule::new("math");
        m.exports.push(ModuleSymbol {
            name: "sin".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Function(1),
        });
        m.exports.push(ModuleSymbol {
            name: "cos".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Function(1),
        });
        loader.register(m).unwrap();

        let imports = loader.resolve_imports("math", &["sin".into(), "cos".into()]);
        assert!(imports.is_ok());
        assert_eq!(imports.unwrap().len(), 2);
    }

    #[test]
    fn loader_resolve_imports_private_error() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let mut m = CompiledModule::new("math");
        m.exports.push(ModuleSymbol {
            name: "_helper".into(),
            vis: Visibility::Private,
            kind: SymbolKind::Function(0),
        });
        loader.register(m).unwrap();

        let result = loader.resolve_imports("math", &["_helper".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("private"));
    }

    #[test]
    fn loader_resolve_imports_not_found() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        loader.register(CompiledModule::new("math")).unwrap();
        let result = loader.resolve_imports("math", &["nonexistent".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }

    #[test]
    fn loader_resolve_imports_module_not_found() {
        let loader = ModuleLoader::new(alloc::vec![]);
        let result = loader.resolve_imports("nonexistent", &["foo".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }

    #[test]
    fn loader_circular_dependency_detection() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let mut a = CompiledModule::new("a");
        a.dependencies.push("b".into());
        let mut b = CompiledModule::new("b");
        b.dependencies.push("c".into());

        loader.register(a).unwrap();
        loader.register(b).unwrap();

        // Now try to register c → a (creates cycle)
        let mut c = CompiledModule::new("c");
        c.dependencies.push("a".into());
        let result = loader.register(c);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Circular"));
    }

    #[test]
    fn loader_begin_finish_loading() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        assert!(loader.begin_loading("silk.graph").is_ok());
        assert!(loader.begin_loading("math").is_ok());
        // Trying to load silk.graph again → cycle
        let result = loader.begin_loading("silk.graph");
        assert!(result.is_err());
        loader.finish_loading("silk.graph");
        // Now it should work
        assert!(loader.begin_loading("silk.graph").is_ok());
    }

    #[test]
    fn loader_would_cycle() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let mut a = CompiledModule::new("a");
        a.dependencies.push("b".into());
        loader.register(a).unwrap();
        assert!(loader.would_cycle("b", "a"));
        assert!(!loader.would_cycle("a", "c"));
    }

    #[test]
    fn loader_load_order() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let mut app = CompiledModule::new("app");
        app.dependencies.push("silk".into());
        app.dependencies.push("context".into());
        let mut silk = CompiledModule::new("silk");
        silk.dependencies.push("olang".into());
        loader.register(CompiledModule::new("olang")).unwrap();
        loader.register(silk).unwrap();
        loader.register(CompiledModule::new("context")).unwrap();
        loader.register(app).unwrap();

        let order = loader.load_order().unwrap();
        let pos = |n: &str| order.iter().position(|x| x == n).unwrap();
        assert!(pos("olang") < pos("silk"), "olang should come before silk");
        assert!(pos("silk") < pos("app"), "silk should come before app");
    }

    // ── CompiledModule tests ────────────────────────────────────────────

    #[test]
    fn compiled_module_exports() {
        let mut m = CompiledModule::new("test");
        m.exports.push(ModuleSymbol {
            name: "pub_fn".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Function(2),
        });
        m.exports.push(ModuleSymbol {
            name: "priv_fn".into(),
            vis: Visibility::Private,
            kind: SymbolKind::Function(0),
        });
        assert!(m.has_export("pub_fn"));
        assert!(!m.has_export("priv_fn")); // private = not exported
        assert_eq!(m.public_symbols(), alloc::vec!["pub_fn"]);
    }

    #[test]
    fn compiled_module_get_export() {
        let mut m = CompiledModule::new("test");
        m.exports.push(ModuleSymbol {
            name: "MyStruct".into(),
            vis: Visibility::Public,
            kind: SymbolKind::Struct(alloc::vec!["x".into(), "y".into()]),
        });
        let sym = m.get_export("MyStruct").unwrap();
        assert_eq!(sym.kind, SymbolKind::Struct(alloc::vec!["x".into(), "y".into()]));
    }

    // ── A13: extract_exports tests ──────────────────────────────────────

    #[test]
    fn extract_exports_pub_fn() {
        let source = "pub fn add(a, b) { a + b; }\nfn helper() { 1; }";
        let stmts = crate::syntax::parse(source).unwrap();
        let exports = extract_exports(&stmts);
        assert_eq!(exports.len(), 2);
        // pub fn → Public
        let add = exports.iter().find(|s| s.name == "add").unwrap();
        assert_eq!(add.vis, Visibility::Public);
        assert_eq!(add.kind, SymbolKind::Function(2));
        // non-pub fn → Private
        let helper = exports.iter().find(|s| s.name == "helper").unwrap();
        assert_eq!(helper.vis, Visibility::Private);
    }

    #[test]
    fn extract_exports_pub_struct() {
        let source = "pub struct Point { x, y }\nstruct Internal { data }";
        let stmts = crate::syntax::parse(source).unwrap();
        let exports = extract_exports(&stmts);
        let point = exports.iter().find(|s| s.name == "Point").unwrap();
        assert_eq!(point.vis, Visibility::Public);
        assert!(matches!(&point.kind, SymbolKind::Struct(f) if f.len() == 2));
        let internal = exports.iter().find(|s| s.name == "Internal").unwrap();
        assert_eq!(internal.vis, Visibility::Private);
    }

    #[test]
    fn extract_exports_pub_enum() {
        let source = "pub enum Color { Red, Green, Blue }";
        let stmts = crate::syntax::parse(source).unwrap();
        let exports = extract_exports(&stmts);
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].vis, Visibility::Public);
        assert!(matches!(&exports[0].kind, SymbolKind::Enum(v) if v.len() == 3));
    }

    #[test]
    fn extract_exports_mixed() {
        let source = "pub fn api() { 1; }\nlet x = 5;\npub let VERSION = 1;";
        let stmts = crate::syntax::parse(source).unwrap();
        let exports = extract_exports(&stmts);
        let api = exports.iter().find(|s| s.name == "api").unwrap();
        assert_eq!(api.vis, Visibility::Public);
        let x = exports.iter().find(|s| s.name == "x").unwrap();
        assert_eq!(x.vis, Visibility::Private);
        let ver = exports.iter().find(|s| s.name == "VERSION").unwrap();
        assert_eq!(ver.vis, Visibility::Public);
    }

    // ── A13: load_from_source tests ─────────────────────────────────────

    #[test]
    fn load_from_source_basic() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let source = "pub fn greet(name) { name; }\nfn internal() { 1; }";
        let result = loader.load_from_source("greeting", source, None);
        assert!(result.is_ok());
        let pub_names = result.unwrap();
        assert!(pub_names.contains(&"greet".to_string()));
        assert!(!pub_names.contains(&"internal".to_string()));
        assert!(loader.is_loaded("greeting"));
    }

    #[test]
    fn load_from_source_cached() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let source = "pub fn f() { 1; }";
        loader.load_from_source("mod_a", source, None).unwrap();
        // Loading again returns cached result
        let result = loader.load_from_source("mod_a", source, None);
        assert!(result.is_ok());
    }

    #[test]
    fn load_from_source_circular_detection() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        // Manually put "b" on loading stack to simulate it's being loaded
        loader.begin_loading("b").unwrap();
        // Now try to load "b" again (simulates circular import)
        let result = loader.load_from_source("b", "pub fn f() { 1; }", Some("a"));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Circular"));
        loader.finish_loading("b");
    }

    #[test]
    fn load_from_source_with_dep_tracking() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let source_a = "pub fn do_a() { 1; }";
        let source_b = "pub fn do_b() { 1; }";
        loader.load_from_source("mod_a", source_a, None).unwrap();
        // mod_b depends on mod_a (requester = "app" loading mod_b)
        loader.load_from_source("mod_b", source_b, Some("app")).unwrap();
        // No cycle — this is fine
        assert!(loader.is_loaded("mod_a"));
        assert!(loader.is_loaded("mod_b"));
    }

    #[test]
    fn load_from_source_selective_import_validation() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let source = "pub fn api() { 1; }\nfn secret() { 2; }";
        loader.load_from_source("mymod", source, None).unwrap();
        // Public symbol → ok
        let result = loader.resolve_imports("mymod", &["api".into()]);
        assert!(result.is_ok());
        // Private symbol → error
        let result = loader.resolve_imports("mymod", &["secret".into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("private"));
    }

    #[test]
    fn load_from_source_parse_error() {
        let mut loader = ModuleLoader::new(alloc::vec![]);
        let bad_source = "fn {{{ invalid";
        let result = loader.load_from_source("bad", bad_source, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Parse error"));
    }

    #[test]
    fn resolve_path_examples() {
        assert_eq!(ModuleLoader::resolve_path("silk.graph"), "silk/graph.ol");
        assert_eq!(ModuleLoader::resolve_path("std.collections"), "std/collections.ol");
        assert_eq!(ModuleLoader::resolve_path("math"), "math.ol");
    }
}
