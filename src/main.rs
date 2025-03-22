#![allow(dead_code)]
use std::collections::HashSet;

use ::ghost_cell::*;
use bumpalo::Bump;

#[derive(Clone, Copy)]
struct PortRef<'graph, 'brand> {
  index: usize,
  node: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
}

impl<'graph, 'brand> PortRef<'graph, 'brand> {
  fn new(node: &'graph GhostCell<'brand, Agent<'graph, 'brand>>, index: usize) -> Self {
    Self { index, node }
  }

  fn connect(token: &mut GhostToken<'brand>, lhs: Option<Self>, rhs: Option<Self>) {
    if let Some(lhs) = lhs {
      lhs.node.borrow_mut(token).connected_to[lhs.index] = rhs;
    }

    if let Some(rhs) = rhs {
      rhs.node.borrow_mut(token).connected_to[rhs.index] = lhs;
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentType {
  Constructor,
  Duplicator,
  Eraser,
}

#[derive(Clone)]
struct Agent<'graph, 'brand> {
  agent_type: AgentType,
  connected_to: [Option<PortRef<'graph, 'brand>>; 3],
}

impl<'graph, 'brand> Agent<'graph, 'brand> {
  fn constructor() -> Agent<'graph, 'brand> {
    Self {
      agent_type: AgentType::Constructor,
      connected_to: [None; 3],
    }
  }

  fn duplicator() -> Agent<'graph, 'brand> {
    Self {
      agent_type: AgentType::Duplicator,
      connected_to: [None; 3],
    }
  }

  fn eraser() -> Agent<'graph, 'brand> {
    Self {
      agent_type: AgentType::Eraser,
      connected_to: [None; 3],
    }
  }
}

fn reduce_local<'graph, 'brand>(
  allocator: &'graph mut Bump,
  permission: &mut GhostToken<'brand>,
  lhs: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
  rhs: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
) {
  assert_eq!(
    lhs.borrow(permission).connected_to[0]
      .expect("This function expects a pair of reducible agents.")
      .node
      .as_ptr(),
    rhs.as_ptr()
  );
  assert_eq!(
    lhs.borrow(permission).connected_to[0]
      .expect("This function expects a pair of reducible agents.")
      .index,
    0
  );
  assert_eq!(
    rhs.borrow(permission).connected_to[0]
      .expect("This function expects a pair of reducible agents.")
      .node
      .as_ptr(),
    lhs.as_ptr()
  );
  assert_eq!(
    rhs.borrow(permission).connected_to[0]
      .expect("This function expects a pair of reducible agents.")
      .index,
    0
  );

  match (
    lhs.borrow(permission).agent_type,
    rhs.borrow(permission).agent_type,
  ) {
    (AgentType::Constructor, AgentType::Constructor) => {
      annihilate(permission, lhs, rhs);
    }
    (AgentType::Duplicator, AgentType::Duplicator) => {
      swap(permission, lhs, rhs);
    }
    (AgentType::Constructor, AgentType::Duplicator)
    | (AgentType::Duplicator, AgentType::Constructor) => {
      duplicate(allocator, permission, lhs, rhs);
    }
    (AgentType::Eraser, AgentType::Constructor) | (AgentType::Eraser, AgentType::Duplicator) => {
      erase(allocator, permission, lhs, rhs);
    }
    (AgentType::Constructor, AgentType::Eraser) | (AgentType::Duplicator, AgentType::Eraser) => {
      erase(allocator, permission, rhs, lhs);
    }
    (AgentType::Eraser, AgentType::Eraser) => {}
  }
}

fn list_nodes<'graph, 'brand>(
  permission: &mut GhostToken<'brand>,
  known_nodes: &mut Vec<&'graph GhostCell<'brand, Agent<'graph, 'brand>>>,
) {
  let root = known_nodes
    .last()
    .expect("list_nodes expects Vec with len > 0");
  let nodes_to_visit: Vec<&'graph GhostCell<'brand, Agent<'graph, 'brand>>> = root
    .borrow(permission)
    .connected_to
    .iter()
    .filter_map(|port| *port)
    .map(|port| port.node)
    .filter(|node| {
      known_nodes
        .iter()
        .find(|x| node as *const _ == *x as *const _)
        .is_none()
    })
    .collect();

  for node in nodes_to_visit {
    known_nodes.push(node);
    list_nodes(permission, known_nodes);
  }
}

fn reduce_global<'graph, 'brand>(
  allocator: &mut Bump,
  permission: &mut GhostToken<'brand>,
  root: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
) {
  let mut nodes = vec![root];
  list_nodes(permission, &mut nodes);
  nodes.retain(|node| {
    if let Some(principal_port) = node.borrow(permission).connected_to[0] {
      principal_port.index == 0 && principal_port.node as *const _ != *node as *const _
    } else {
      false
    }
  });

  for node in nodes.iter() {
    reduce_local(
      allocator,
      permission,
      node,
      node.borrow(permission).connected_to[0].unwrap().node,
    )
  }
}

fn annihilate<'graph, 'brand>(
  permission: &mut GhostToken<'brand>,
  lhs: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
  rhs: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
) {
  let port_a = lhs.borrow(permission).connected_to[1];
  let port_b = lhs.borrow(permission).connected_to[2];
  let port_c = rhs.borrow(permission).connected_to[1];
  let port_d = rhs.borrow(permission).connected_to[2];
  PortRef::connect(permission, port_a, port_c);
  PortRef::connect(permission, port_b, port_d);
}

fn swap<'graph, 'brand>(
  permission: &mut GhostToken<'brand>,
  lhs: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
  rhs: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
) {
  let port_a = lhs.borrow(permission).connected_to[1];
  let port_b = lhs.borrow(permission).connected_to[2];
  let port_c = rhs.borrow(permission).connected_to[1];
  let port_d = rhs.borrow(permission).connected_to[2];
  PortRef::connect(permission, port_a, port_d);
  PortRef::connect(permission, port_b, port_c);
}

fn duplicate<'graph, 'brand>(
  allocator: &'graph mut Bump,
  permission: &mut GhostToken<'brand>,
  top_right: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
  bottom_left: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
) {
  let bottom_right = allocator.alloc(GhostCell::new(top_right.borrow(permission).clone()));
  let top_left = allocator.alloc(GhostCell::new(top_right.borrow(permission).clone()));

  let port5 = top_left.borrow(permission).connected_to[0];
  let port6 = bottom_left.borrow(permission).connected_to[2];
  PortRef::connect(permission, port5, port6);

  let port7 = bottom_right.borrow(permission).connected_to[0];
  let port8 = top_right.borrow(permission).connected_to[2];
  PortRef::connect(permission, port7, port8);

  let port1 = top_right.borrow(permission).connected_to[0];
  let port2 = bottom_left.borrow(permission).connected_to[1];
  PortRef::connect(permission, port1, port2);

  let port3 = bottom_left.borrow(permission).connected_to[0];
  let port4 = top_right.borrow(permission).connected_to[1];
  PortRef::connect(permission, port3, port4);

  let port9 = Some(PortRef::new(top_left, 1));
  let port10 = Some(PortRef::new(bottom_left, 2));
  PortRef::connect(permission, port9, port10);

  let port15 = Some(PortRef::new(top_right, 2));
  let port16 = Some(PortRef::new(bottom_right, 1));
  PortRef::connect(permission, port15, port16);

  let port11 = Some(PortRef::new(top_left, 2));
  let port12 = Some(PortRef::new(bottom_right, 2));
  PortRef::connect(permission, port11, port12);

  let port13 = Some(PortRef::new(top_right, 1));
  let port14 = Some(PortRef::new(bottom_left, 1));
  PortRef::connect(permission, port13, port14);
}

fn erase<'graph, 'brand>(
  allocator: &'graph mut Bump,
  permission: &mut GhostToken<'brand>,
  eraser: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
  to_be_erased: &'graph GhostCell<'brand, Agent<'graph, 'brand>>,
) {
  let another_eraser = allocator.alloc(GhostCell::new(eraser.borrow(permission).clone()));
  let another_eraser_principal = Some(PortRef::new(another_eraser, 0));
  let port1 = to_be_erased.borrow(permission).connected_to[1];
  PortRef::connect(permission, another_eraser_principal, port1);

  let eraser_principal = Some(PortRef::new(eraser, 0));
  let port2 = to_be_erased.borrow(permission).connected_to[2];
  PortRef::connect(permission, eraser_principal, port2);
}

fn main() {
  println!("Hello, world!");
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn zero() {
    GhostToken::new(|mut token| {
      let eraser = GhostCell::new(Agent::eraser());
      let root = GhostCell::new(Agent::constructor());
      let body = GhostCell::new(Agent::constructor());

      PortRef::connect(
        &mut token,
        Some(PortRef::new(&root, 1)),
        Some(PortRef::new(&eraser, 0)),
      );
      PortRef::connect(
        &mut token,
        Some(PortRef::new(&root, 2)),
        Some(PortRef::new(&body, 0)),
      );
      PortRef::connect(
        &mut token,
        Some(PortRef::new(&body, 1)),
        Some(PortRef::new(&body, 2)),
      );

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
      assert_eq!(root.borrow(&token).connected_to[2].unwrap().index, 0);
      assert_eq!(
        body.borrow(&token).connected_to[0].unwrap().node.as_ptr(),
        root.as_ptr()
      );
      assert_eq!(body.borrow(&token).connected_to[0].unwrap().index, 2);

      assert_eq!(
        body.borrow(&token).connected_to[1].unwrap().node.as_ptr(),
        body.as_ptr()
      );
      assert_eq!(body.borrow(&token).connected_to[1].unwrap().index, 2);
      assert_eq!(
        body.borrow(&token).connected_to[2].unwrap().node.as_ptr(),
        body.as_ptr()
      );
      assert_eq!(body.borrow(&token).connected_to[2].unwrap().index, 1);
    });
  }
}
