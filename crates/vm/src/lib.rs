pub mod bytecode;
pub mod function;
mod scope;
pub mod types;
pub mod value;

pub use bytecode::*;
pub use scope::Scope;
pub use types::Type;
pub use value::Value;

// maybe not great to have this always there even though its only needed during tests
pub mod test_helpers;

// TODO move this to a proper place
use serialize::{Deserialize, DeserializeCtx};
use std::collections::HashMap;
use std::io;
use std::rc::Rc;

pub struct DeserializationContext {
    global: Option<Rc<Scope>>,
    scopes: Vec<Rc<Scope>>,
    idents: HashMap<usize, Rc<str>>,
}

impl DeserializationContext {
    pub fn new(num_scopes: usize, num_idents: usize, global: Option<Rc<Scope>>) -> Self {
        Self {
            scopes: Vec::with_capacity(num_scopes),
            idents: HashMap::with_capacity(num_idents),
            global,
        }
    }

    pub fn add_scope(&mut self, id: usize, parent_id: Option<usize>) {
        // While serializing scopes, we must guarantee that scopes are serialized in order of their
        // id, and so will be deserialized in order of their id
        // While generating scopes, the children blocks always come after their parent, hence their
        // id must be greater that their parent. Thus when adding a child, its parent must already
        // be added
        self.scopes.push(Rc::new(Scope::new(
            id,
            parent_id
                .map(|id| Rc::clone(&self.scopes[id]))
                .or_else(|| self.global.clone()),
        )))
    }

    pub fn get_scope(&self, id: usize) -> Rc<Scope> {
        Rc::clone(&self.scopes[id])
    }

    pub fn add_ident(&mut self, id: usize, ident: Rc<str>) {
        self.idents.insert(id, ident);
    }

    pub fn get_ident(&self, id: usize) -> Rc<str> {
        Rc::clone(&self.idents[&id])
    }
}

impl DeserializeCtx<DeserializationContext> for Rc<str> {
    fn deserialize_with_context<R: io::BufRead>(
        data: &mut R,
        ctx: &mut DeserializationContext,
    ) -> Result<Self, io::Error> {
        let id = usize::deserialize(data)?;
        Ok(ctx.get_ident(id))
    }
}
