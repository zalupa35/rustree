use crate::{
    types::Document,
    utils::{get_node_parent, modify_document, DocumentModification},
};

#[derive(Clone, Default, Debug)]
pub struct HistoryManager {
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

#[derive(Clone, Debug)]
struct Action {
    undo: DocumentModification,
    redo: DocumentModification,
}

fn create_undo_modification(
    document: Document,
    modification: DocumentModification,
) -> DocumentModification {
    match modification.clone() {
        DocumentModification::EditNode(id, _) => {
            DocumentModification::EditNode(id, document.get_node(id).unwrap())
        }
        DocumentModification::DeleteNode(id) => {
            let node = document.clone().get_node(id).unwrap();
            let root = document.root_node;
            let parent = get_node_parent(root.clone(), id);
            DocumentModification::PasteNode(node, parent.unwrap_or(root.id))
        }
        DocumentModification::CreateNode(_, new_id) => DocumentModification::DeleteNode(new_id),
        _ => modification,
    }
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn register_document_modification(
        &mut self,
        document: Document,
        modification: DocumentModification,
    ) -> Document {
        self.undo_stack.push(Action {
            undo: create_undo_modification(document.clone(), modification.clone()),
            redo: modification.clone(),
        });
        modify_document(document, modification)
    }
    pub fn undo_action(&mut self, document: Document) -> Option<Document> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            return Some(modify_document(document, action.undo));
        }
        None
    }
    pub fn redo_action(&mut self, document: Document) -> Option<Document> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            return Some(modify_document(document, action.redo));
        }
        None
    }
}
