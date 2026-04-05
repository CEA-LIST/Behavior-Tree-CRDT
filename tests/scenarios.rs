use behaviortree::{
    classifiers::{
        Action, ActionKind, BehaviorTree, Blackboard, BlackboardEntry, ExecutionNode,
        ExecutionNodeKind, InFlowPort, OpenDoor, Root, TreeNodeKind,
    },
    package::{Behaviortree, BehaviortreeLog},
};
use moirai_crdt::{
    list::{eg_walker::List, nested_list::NestedList},
    utils::membership::twins_log,
};
use moirai_protocol::{crdt::query::Read, replica::IsReplica};

#[test]
fn vertex_cascade_creation() {
    let (mut replica_a, _) = twins_log::<BehaviortreeLog>();

    let _ = replica_a
        .send(Behaviortree::Root(Root::Main(BehaviorTree::Child(
            Box::new(TreeNodeKind::ExecutionNode(ExecutionNodeKind::Action(
                ActionKind::OpenDoor(OpenDoor::ActionSuper(Action::ExecutionNodeSuper(
                    ExecutionNode::Inflowports(NestedList::Insert {
                        pos: 0,
                        op: InFlowPort::New,
                    }),
                ))),
            ))),
        ))))
        .unwrap();

    assert_eq!(replica_a.query(Read::new()).refs.node_count(), 1);
}

/// This test checks that when we update a blackboard entry while another replica deletes it concurrently,
/// the delete do reset the blackboard entry to the default op, but the update is then applied.
/// In addition, we check that the vertex is revived in the reference graph.
#[test]
fn blackboard_update_delete() {
    let (mut replica_a, mut replica_b) = twins_log::<BehaviortreeLog>();

    let a1 = replica_a
        .send(Behaviortree::Root(Root::Main(BehaviorTree::Blackboard(
            Blackboard::Entries(NestedList::Insert {
                pos: 0,
                op: BlackboardEntry::Key(List::Insert {
                    content: 'a',
                    pos: 0,
                }),
            }),
        ))))
        .unwrap();

    replica_b.receive(a1);

    let b1 = replica_b
        .send(Behaviortree::Root(Root::Main(BehaviorTree::Blackboard(
            Blackboard::Entries(NestedList::Update {
                pos: 0,
                op: BlackboardEntry::Key(List::Insert {
                    content: 'z',
                    pos: 1,
                }),
            }),
        ))))
        .unwrap();

    let a2 = replica_a
        .send(Behaviortree::Root(Root::Main(BehaviorTree::Blackboard(
            Blackboard::Entries(NestedList::Delete { pos: 0 }),
        ))))
        .unwrap();

    replica_a.receive(b1);
    replica_b.receive(a2);

    assert_eq!(
        replica_a.query(Read::new()).root,
        replica_b.query(Read::new()).root
    );
    let a_refs = replica_a.query(Read::new()).refs;
    let b_refs = replica_b.query(Read::new()).refs;
    assert_eq!(a_refs.node_count(), b_refs.node_count());
    assert_eq!(a_refs.edge_count(), b_refs.edge_count());
}
