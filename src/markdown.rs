// usaidwat
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Markdown parsers.
//!
//! # See also
//!
//! - [CommonMark](https://commonmark.org/), for the CommonMark specification.

pub mod bare;
pub mod terminal;

pub use bare::parse as summarize;
pub use terminal::parse;

use log::{trace, warn};
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
        trace!("swallowing node: {node:#?}");
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
