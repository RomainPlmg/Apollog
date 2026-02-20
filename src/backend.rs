use crate::{analyser::*, types::*, utils::is_position_in_range};
use dashmap::DashMap;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::*,
    {Client, LanguageServer},
};
use tree_sitter::Tree;

#[derive(Debug, Clone)]
pub struct DocumentData {
    pub text: String,
    pub tree: Tree,
    pub modules: Vec<Module>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    db: DashMap<Url, DocumentData>,
    analyser: Analyser,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: String::from("Apollog"),
                version: Some(String::from("0.0.1")),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.parse_and_store(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(content) = params.content_changes.first() {
            self.parse_and_store(params.text_document.uri, content.text.clone())
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let document = self.db.get(&params.text_document.uri);
        if let Some(doc) = document {
            self.client
                .publish_diagnostics(params.text_document.uri, doc.diagnostics.clone(), None)
                .await;
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        if let Some(document) = self.db.get(&uri) {
            let modules = &document.modules;
            let mut module_symbols = Vec::new();

            for module in modules {
                let mut port_symbols = Vec::new();
                for port in &module.ports {
                    let display_name = if port.name.is_empty() {
                        "<unnamed port>".to_string()
                    } else {
                        port.name.clone()
                    };

                    port_symbols.push({
                        DocumentSymbol {
                            name: display_name,
                            detail: Some(format!(
                                "{:?} {:?} {}",
                                port.direction,
                                port.class,
                                port.size.as_deref().unwrap_or_default()
                            )),
                            kind: SymbolKind::FIELD,
                            tags: None,
                            deprecated: None,
                            range: port.range,
                            selection_range: port.selection_range,
                            children: None,
                        }
                    });
                }

                let display_name = if module.name.is_empty() {
                    "<unnamed module>".to_string()
                } else {
                    module.name.clone()
                };

                module_symbols.push(DocumentSymbol {
                    name: display_name,
                    detail: None,
                    kind: SymbolKind::STRUCT,
                    tags: None,
                    deprecated: None,
                    range: module.range,
                    selection_range: module.range,
                    children: Some(port_symbols),
                });
            }
            return Ok(Some(DocumentSymbolResponse::Nested(module_symbols)));
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(document) = self.db.get(&uri) {
            let modules = &document.modules;
            for module in modules {
                for port in &module.ports {
                    if is_position_in_range(position, port.range) {
                        return Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(format!(
                                "**Port**: {}  \n**Type**: {:?} {:?} {}",
                                port.name,
                                port.direction,
                                port.class,
                                port.size.as_deref().unwrap_or("")
                            ))),
                            range: Some(port.range),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            db: DashMap::new(),
            analyser: Analyser::new(),
        }
    }

    async fn parse_and_store(&self, uri: Url, code: String) {
        let (tree, modules, diagnostics) = self.analyser.parse_file(&code);

        self.client.log_message(MessageType::INFO, format!("{}", tree.root_node().to_sexp())).await;

        let new_doc = DocumentData {
            text: code,
            modules: modules,
            tree: tree,
            diagnostics,
        };

        self.db.insert(uri, new_doc);
    }
}
