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

  fn connect(self, other: Self, token: &mut GhostToken<'brand>) {
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

  #[test]
  fn zero() {
    GhostToken::new(|mut token| {
      let eraser = GhostCell::new(Agent::eraser());
      let root = GhostCell::new(Agent::constructor());
      let body = GhostCell::new(Agent::constructor());

      PortRef::new(&root, 1).connect(PortRef::new(&eraser, 0), &mut token);
      PortRef::new(&root, 2).connect(PortRef::new(&body, 0), &mut token);

      assert_eq!(
        root.borrow(&token).connected_to[1].unwrap().node.as_ptr(),
        eraser.as_ptr()
      );
      assert_eq!(root.borrow(&token).connected_to[1].unwrap().index, 0);
      assert_eq!(
        eraser.borrow(&token).connected_to[0].unwrap().node.as_ptr(),
        root.as_ptr()
      );
      assert_eq!(eraser.borrow(&token).connected_to[0].unwrap().index, 1);

      assert_eq!(
        root.borrow(&token).connected_to[2].unwrap().node.as_ptr(),
        body.as_ptr()
      );
      assert_eq!(root.borrow(&token).connected_to[1].unwrap().index, 0);
      assert_eq!(
        body.borrow(&token).connected_to[0].unwrap().node.as_ptr(),
        root.as_ptr()
      );
      assert_eq!(body.borrow(&token).connected_to[0].unwrap().index, 2);
    });
  }
}
