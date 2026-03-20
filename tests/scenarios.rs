use behaviortree::{
    classifiers::{BehaviorTree, Blackboard, BlackboardEntry, Root},
    package::{Behaviortree, BehaviortreeLog},
    references::{Instance, Ref},
};
use moirai_crdt::list::{eg_walker::List, nested_list::NestedList};
use moirai_protocol::{
    broadcast::tcsb::Tcsb,
    crdt::query::Read,
    replica::{IsReplica, Replica},
};

fn graph_vertices(replica: &Replica<BehaviortreeLog, Tcsb<Behaviortree>>) -> Vec<String> {
    let mut vertices = replica
        .query(Read::new())
        .refs
        .node_weights()
        .map(|i| match i {
            Instance::BlackboardEntryId(id) => format!("BlackboardEntryId({})", id.0),
            Instance::OutFlowPortId(id) => format!("OutFlowPortId({})", id.0),
            Instance::InFlowPortId(id) => format!("InFlowPortId({})", id.0),
        })
        .collect::<Vec<_>>();
    vertices.sort();
    vertices
}

fn graph_edges(replica: &Replica<BehaviortreeLog, Tcsb<Behaviortree>>) -> Vec<String> {
    let mut edges = replica
        .query(Read::new())
        .refs
        .edge_weights()
        .map(|e| match e {
            Ref::OutFlowPortToBlackboardEntry(e) => format!("{:?}", e),
            Ref::InFlowPortToBlackboardEntry(e) => format!("{:?}", e),
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
}

/// This test checks that when we update a blackboard entry while another replica deletes it concurrently,
/// the delete do reset the blackboard entry to the default value, but the update is then applied.
/// In addition, we check that the vertex is revived in the reference graph.
#[test]
fn blackboard_update_delete() {
    let mut replica_a = Replica::<BehaviortreeLog, Tcsb<Behaviortree>>::new("a".to_string());
    let mut replica_b = Replica::<BehaviortreeLog, Tcsb<Behaviortree>>::new("b".to_string());

    let a1 = replica_a
        .send(Behaviortree::Root(Root::Main(BehaviorTree::Blackboard(
            Blackboard::Entries(NestedList::Insert {
                pos: 0,
                value: BlackboardEntry::Key(List::Insert {
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
                value: BlackboardEntry::Key(List::Insert {
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

    println!("Replica A, refs, nodes : {:?}", graph_vertices(&replica_a));
    println!("Replica B, refs, nodes : {:?}", graph_vertices(&replica_b));
    println!("Replica A, refs, edges : {:?}", graph_edges(&replica_a));
    println!("Replica B, refs, edges : {:?}", graph_edges(&replica_b));

    println!("{:#?}", replica_a.query(Read::new()));

    assert_eq!(
        replica_a.query(Read::new()).root,
        replica_b.query(Read::new()).root
    );
    let a_refs = replica_a.query(Read::new()).refs;
    let b_refs = replica_b.query(Read::new()).refs;
    assert_eq!(a_refs.node_count(), b_refs.node_count());
    assert_eq!(a_refs.edge_count(), b_refs.edge_count());
    assert_eq!(graph_vertices(&replica_a), graph_vertices(&replica_b));
}

#[test]
fn fuzz() {
    use moirai_fuzz::{
        config::{FuzzerConfig, RunConfig},
        fuzzer::fuzzer,
    };

    let run = RunConfig::new(0.6, 8, 100, None, None, true, false);
    let runs = vec![run.clone(); 10];

    let config = FuzzerConfig::<BehaviortreeLog>::new(
        "bt",
        runs,
        true,
        |a, b| {
            a.refs.node_count() == b.refs.node_count()
                && a.refs.edge_count() == b.refs.edge_count()
                && a.root == b.root
        },
        false,
    );

    fuzzer::<BehaviortreeLog>(config);
}
