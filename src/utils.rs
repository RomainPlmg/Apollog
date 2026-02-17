use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn ts_to_lsp_range(node: tree_sitter::Node) -> Range {
    Range {
        start: Position::new(
            node.start_position().row as u32,
            node.start_position().column as u32,
        ),
        end: Position::new(
            node.end_position().row as u32,
            node.end_position().column as u32,
        ),
    }
}

pub fn collect_errors(node: tree_sitter::Node, diagnostics: &mut Vec<Diagnostic>) {
    if node.is_error() || node.is_missing() {
        let range = ts_to_lsp_range(node);
        let message = if node.is_missing() {
            format!("Incomplete syntaxe: expect '{}'", node.kind())
        } else {
            "Syntax error".to_string()
        };

        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("Apollog".to_string()),
            message,
            ..Default::default()
        });
    }

    // Recursive function
    if node.has_error() {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_errors(child, diagnostics);
        }
    }
}

pub fn is_position_in_range(pos: Position, range: Range) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }
    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }
    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }
    true
}
