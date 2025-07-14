//! Markdown parsers.
//!
//! # See also
//!
//! - [CommonMark](https://commonmark.org/), for the CommonMark specification.

pub mod bare;
pub mod terminal;

pub use bare::parse as summarize;
pub use terminal::parse;

use log::{debug, warn};
use markdown::mdast::Node;

/// "Visit" a node and emit code.
///
/// For example, a `Visitor` can embody an algorithm used to visit each node
/// in an abstract syntax and emit code for the tree.
pub trait Visitor {
    /// The generated text.
    fn text(&self) -> String;

    /// "Visit" a particular node in a graph.
    fn visit(&mut self, node: &Node);

    /// "Swallows" a node.
    ///
    /// Nothing is done by the visitor for the node, but it continues visiting
    /// the node's children. This is generally used to accept a known node
    /// when there is no further processing necessary for that particular node.
    fn swallow(&mut self, node: &Node)
    where
        Self: Sized,
    {
        debug!("swallowing node: {node:#?}");
        node.accept_children(self);
    }

    /// Indicates that the visitor was asked to visit an unexpected node
    /// that it does not know how to process.
    ///
    /// By default, the visitor will print an error and continue.
    fn unknown(&self, node: &Node) {
        warn!("unhandled node: {node:#?}");
    }
}

/// A data structure that can be visited.
pub trait Visitable {
    /// Accept a visitor for processing the visitable item.
    fn accept<V: Visitor>(&self, visitor: &mut V);

    /// Accept a visitor for processing all child nodes.
    fn accept_children<V: Visitor>(&self, visitor: &mut V);
}

impl Visitable for Node {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit(self);
    }

    fn accept_children<V: Visitor>(&self, visitor: &mut V) {
        if let Some(children) = self.children() {
            for child in children {
                child.accept(visitor);
            }
        }
    }
}

/// A data type that can append text.
trait TextAppendable {
    /// Appends `text` to the target data structure.
    fn push_text(&mut self, text: &str);
}

#[cfg(test)]
mod test_utils;
