use std::collections::HashSet;

use crate::spec::Spec;

#[derive(Default)]
pub struct Names {
  root:         HashSet<String>,
  notification: HashSet<String>,
  request:      HashSet<String>,
}

impl Names {
  pub fn from_spec(spec: &Spec) -> Self {
    let mut names = Names::default();

    names.notification.extend(spec.notifications.iter().map(|n| crate::rpc_name(&n.method)));
    names.request.extend(spec.notifications.iter().map(|r| crate::rpc_name(&r.method)));

    names.root.extend(spec.structures.iter().map(|ty| ty.name.clone()));
    names.root.extend(spec.enumerations.iter().map(|ty| ty.name.clone()));
    names.root.extend(spec.type_aliases.iter().map(|ty| ty.name.clone()));

    names
  }

  pub fn resolve(&self, name: &str) -> String {
    if self.request.contains(name) {
      format!("crate::request::{name}")
    } else if self.notification.contains(name) {
      format!("crate::notification::{name}")
    } else if self.root.contains(name) {
      format!("crate::{name}")
    } else {
      match name {
        "CodeActionRequest" => "crate::request::TextDocumentCodeAction",
        "CodeLensRequest" => "crate::request::TextDocumentCodeLens",
        "CodeLensResolveRequest" => "crate::request::CodeLensResolve",
        "ColorPresentationRequest" => "crate::request::TextDocumentColorPresentation",
        "CompletionRequest" => "crate::request::TextDocumentCompletion",
        "CompletionResolveRequest" => "crate::request::CompletionItemResolve",
        "DefinitionRequest" => "crate::request::TextDocumentDefinition",
        "DocumentColorRequest" => "crate::request::TextDocumentDocumentColor",
        "DocumentFormattingRequest" => "crate::request::TextDocumentFormatting",
        "DocumentHighlightRequest" => "crate::request::TextDocumentDocumentHighlight",
        "DocumentLinkRequest" => "crate::request::TextDocumentDocumentLink",
        "DocumentOnTypeFormattingRequest" => "crate::request::TextDocumentOnTypeFormatting",
        "DocumentRangeFormattingRequest" => "crate::request::TextDocumentRangeFormatting",
        "DocumentRangesFormattingRequest" => "crate::request::TextDocumentRangesFormatting",
        "DocumentSymbolRequest" => "crate::request::TextDocumentDocumentSymbol",
        "ExecuteCommandRequest" => "crate::request::WorkspaceExecuteCommand",
        "FoldingRangeList" => "crate::FoldingRange",
        "FoldingRangeRequest" => "crate::request::TextDocumentFoldingRange",
        "HoverRequest" => "crate::request::TextDocumentHover",
        "InlayHintsParams" => "InlayHintParams",
        "InlineCompletion" => "InlineCompletionItem",
        "ReferencesRequest" => "crate::request::TextDocumentReferences",
        "RenameRequest" => "crate::request::TextDocumentRename",
        "ResponseError" => "",
        "SignatureHelpRequest" => "crate::request::TextDocumentSignatureHelp",
        "TextDocumentPosition" => "TextDocumentPositionParams",
        "WorkspaceSymbolRequest" => "crate::request::WorkspaceSymbolRequest",

        s if s.starts_with("CallHierarchyIncomingCall") => "crate::CallHierarchyIncomingCall",
        s if s.starts_with("CallHierarchyItem") => "crate::CallHierarchyItem",
        s if s.starts_with("CallHierarchyItemProvider") => "crate::CallHierarchyItemProvider",
        s if s.starts_with("CallHierarchyOutgoingCall") => "crate::CallHierarchyOutgoingCall",
        s if s.starts_with("CodeActionProvider") => "crate::request::TextDocumentCodeAction",
        s if s.starts_with("ColorPresentation") => "crate::ColorPresentation",
        s if s.starts_with("CompletionItem") => "crate::CompletionItem",
        s if s.starts_with("DocumentHighlightKind") => "crate::DocumentHighlightKind",
        s if s.starts_with("InlineCompletionItem") => "crate::InlineCompletionItem",
        s if s.starts_with("TextDocument") => "crate::TextDocumentIdentifier",
        s if s.starts_with("TypeHierarchyItem") => "crate::TypeHierarchyItem",
        s if s.starts_with("Uri") => "String", // TODO: Uri type.

        _ => panic!("unresolved name: {name}"),
      }
      .into()
    }
  }
}
