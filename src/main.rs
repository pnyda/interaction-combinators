use std::{marker::PhantomData, ops::Index};

use ::ghost_cell::*;

#[derive(Clone, Copy)]
struct PortRef<'graph, 'brand> {
  index: usize,
  node: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
}

impl<'graph, 'brand> PortRef<'graph, 'brand> {
  fn new(node: &'graph GhostCell<'brand, Agent<'graph, 'brand>>, index: usize) -> Self {
    Self { index, node }
  }

  fn connect<'a>(self, other: Self, token: &'a mut GhostToken<'brand>) {
    self.node.borrow_mut(token).connected_to[self.index].replace(other);
    other.node.borrow_mut(token).connected_to[other.index].replace(self);
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentType {
  Constructor,
  Duplicator,
  Eraser,
}

#[derive(Clone)]
struct Agent<'a, 'b> {
  agent_type: AgentType,
  connected_to: [Option<PortRef<'a, 'b>>; 3],
}

impl<'a, 'b> Agent<'a, 'b> {
  fn constructor() -> Agent<'a, 'b> {
    Self {
      agent_type: AgentType::Constructor,
      connected_to: [None; 3],
    }
  }

  fn duplicator() -> Agent<'a, 'b> {
    Self {
      agent_type: AgentType::Duplicator,
      connected_to: [None; 3],
    }
  }

  fn eraser() -> Agent<'a, 'b> {
    Self {
      agent_type: AgentType::Eraser,
      connected_to: [None; 3],
    }
  }
}

fn main() {
  println!("Hello, world!");
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::ghost_cell::*;
  use bumpalo::Bump;

  #[test]
  fn zero() {
    GhostToken::new(|mut token| {
      let allocator = Bump::new();
      let eraser = allocator.alloc(GhostCell::new(Agent::eraser()));
      let root = allocator.alloc(GhostCell::new(Agent::constructor()));
      let body = allocator.alloc(GhostCell::new(Agent::constructor()));
      PortRef::new(root, 1).connect(PortRef::new(eraser, 0), &mut token);
      PortRef::new(root, 2).connect(PortRef::new(body, 0), &mut token);

      // assert!(eraser.borrow(&token).connected_to[0].unwrap().node.as_ptr() == root.as_ptr());
      // assert!(eraser.borrow(&token).connected_to[0].unwrap().index == 1);
    });
  }

  #[test]
  fn zero2() {
    GhostToken::new(|mut token| {
      let eraser = GhostCell::new(Agent::eraser());
      let root = GhostCell::new(Agent::constructor());
      let body = GhostCell::new(Agent::constructor());
      PortRef::new(&root, 1).connect(PortRef::new(&eraser, 0), &mut token);
    });
  }
}
