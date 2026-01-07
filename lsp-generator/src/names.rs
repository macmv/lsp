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

    names.notification.extend(spec.notifications.iter().map(|n| crate::to_pascal_case(&n.method)));
    names.request.extend(spec.notifications.iter().map(|r| crate::to_pascal_case(&r.method)));

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
        "CodeActionRequest" => "crate::CodeActionRequest",
        "CodeLensRequest" => "crate::CodeLensRequest",
        "CodeLensResolveRequest" => "crate::CodeLensResolveRequest",
        "ColorPresentationRequest" => "crate::ColorPresentationRequest",
        "CompletionRequest" => "crate::CompletionRequest",
        "CompletionResolveRequest" => "crate::CompletionResolveRequest",
        "DefinitionRequest" => "crate::DefinitionRequest",
        "DocumentColorRequest" => "crate::request::DocumentColor",
        "DocumentFormattingRequest" => "crate::DocumentFormattingRequest",
        "DocumentHighlightRequest" => "crate::DocumentHighlightRequest",
        "DocumentLinkRequest" => "crate::DocumentLinkRequest",
        "DocumentOnTypeFormattingRequest" => "crate::DocumentOnTypeFormattingRequest",
        "DocumentRangeFormattingRequest" => "crate::DocumentRangeFormattingRequest",
        "DocumentRangesFormattingRequest" => "crate::DocumentRangesFormattingRequest",
        "DocumentSymbolRequest" => "crate::DocumentSymbolRequest",
        "ExecuteCommandRequest" => "crate::ExecuteCommandRequest",
        "FoldingRangeList" => "FoldingRange",
        "FoldingRangeRequest" => "crate::FoldingRangeRequest",
        "HoverRequest" => "crate::HoverRequest",
        "InlayHintsParams" => "InlayHintParams",
        "InlineCompletion" => "InlineCompletionItem",
        "ReferencesRequest" => "crate::ReferencesRequest",
        "RenameRequest" => "crate::RenameRequest",
        "ResponseError" => "crate::ResponseError",
        "SignatureHelpRequest" => "crate::SignatureHelpRequest",
        "TextDocumentPosition" => "TextDocumentPositionParams",
        "WorkspaceSymbolRequest" => "crate::WorkspaceSymbolRequest",

        s if s.starts_with("CallHierarchyIncomingCall") => "crate::CallHierarchyIncomingCall",
        s if s.starts_with("CallHierarchyItem") => "crate::CallHierarchyItem",
        s if s.starts_with("CallHierarchyItemProvider") => "crate::CallHierarchyItemProvider",
        s if s.starts_with("CallHierarchyOutgoingCall") => "crate::CallHierarchyOutgoingCall",
        s if s.starts_with("ColorPresentation") => "crate::ColorPresentation",
        s if s.starts_with("CompletionItem") => "crate::CompletionItem",
        s if s.starts_with("DocumentHighlightKind") => "crate::DocumentHighlightKind",
        s if s.starts_with("InlineCompletionItem") => "crate::InlineCompletionItem",
        s if s.starts_with("TextDocument") => "crate::TextDocument",
        s if s.starts_with("TypeHierarchyItem") => "crate::TypeHierarchyItem",
        s if s.starts_with("Uri") => "crate::Uri",

        _ => panic!("unresolved name: {name}"),
      }
      .into()
    }
  }
}
