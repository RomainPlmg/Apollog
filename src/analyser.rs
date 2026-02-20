use tower_lsp::lsp_types::Diagnostic;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator, Tree};

use crate::{types::*, utils::*};

#[derive(Debug)]
pub struct Analyser {
    query: Query,
}

impl Analyser {
    pub fn new() -> Self {
        let language = tree_sitter_verilog::LANGUAGE.into();
        let query = Query::new(&language, include_str!("queries/symbols.scm"))
            .expect("Invalid query syntax");
        Self { query }
    }

    pub fn parse_file(&self, code: &str) -> (Tree, Vec<Module>, Vec<Diagnostic>) {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_verilog::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();

        let mut diagnostics = Vec::new();
        collect_errors(tree.clone().root_node(), &mut diagnostics);

        let modules = self.extract_symbols(tree.clone(), code);
        (tree, modules, diagnostics)
    }

    fn extract_symbols(&self, tree: Tree, code: &str) -> Vec<Module> {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), code.as_bytes());

        let mut all_modules: Vec<Module> = Vec::new();
        let mut current_module: Option<Module> = None;
        let mut pending_port = Port::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let name = self.query.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(code.as_bytes()).unwrap_or("");

                match name {
                    "module.name" => {
                        if let Some(m) = current_module.take() {
                            all_modules.push(m);
                        }
                        let mut m = Module::default();
                        m.name = text.to_string();
                        m.range = ts_to_lsp_range(node);
                        current_module = Some(m);
                    }
                    "port.name" => {
                        pending_port.name = text.to_string();
                        pending_port.selection_range = ts_to_lsp_range(node);
                        if let Some(ref mut m) = current_module {
                            m.ports.push(pending_port);
                            pending_port = Port::new();
                        }
                    }
                    "port.class" => {
                        pending_port.class = match text {
                            "reg" => PortType::Reg,
                            _ => PortType::Wire,
                        }
                    }
                    "port.direction" => {
                        pending_port.direction = match text {
                            "input" => PortDir::Input,
                            "output" => PortDir::Output,
                            _ => PortDir::Inout,
                        }
                    }
                    "port.size" => {
                        pending_port.size = Some(text.to_string());
                    }
                    "port.item" => {
                        pending_port.range = ts_to_lsp_range(node);
                    }
                    _ => {}
                }
            }
        }

        if let Some(m) = current_module {
            all_modules.push(m);
        }

        all_modules
    }
}
