use tower_lsp::lsp_types::Range;

#[derive(Debug, Clone, Copy)]
pub enum PortDir {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, Copy)]
pub enum PortType {
    Wire,
    Reg,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub class: PortType,
    pub direction: PortDir,
    pub size: Option<String>,
    pub range: Range,
    pub selection_range: Range,
}

#[derive(Debug, Default, Clone)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub range: Range,
}

impl Port {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            class: PortType::Wire,
            direction: PortDir::Inout,
            size: None,
            range: Range::default(),
            selection_range: Range::default(),
        }
    }
}
