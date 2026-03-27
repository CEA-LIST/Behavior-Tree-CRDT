use behaviortree::{
    classifiers::{
        Action, ActionFeat, BehaviorTree, Blackboard, BlackboardEntry, ExecutionNode,
        ExecutionNodeFeat, InFlowPort, OpenDoor, Root, TreeNode,
    },
    package::{Behaviortree, BehaviortreeLog},
    references::{Instance, Ref},
};
use moirai_crdt::{
    list::{eg_walker::List, nested_list::NestedList},
    utils::membership::twins_log,
};
use moirai_protocol::{
    broadcast::tcsb::Tcsb,
    crdt::query::Read,
    replica::{self, IsReplica, Replica},
};
use petgraph::{Direction, graph::DiGraph};

struct Vf2GraphView<'a>(&'a DiGraph<Instance, behaviortree::references::Ref>);

impl<'a> vf2::Graph for Vf2GraphView<'a> {
    type NodeLabel = Instance;
    type EdgeLabel = behaviortree::references::Ref;

    fn is_directed(&self) -> bool {
        true
    }

    fn node_count(&self) -> usize {
        self.0.node_count()
    }

    fn node_label(&self, node: vf2::NodeIndex) -> Option<&Self::NodeLabel> {
        self.0.node_weight(petgraph::graph::NodeIndex::new(node))
    }

    fn neighbors(
        &self,
        node: vf2::NodeIndex,
        direction: vf2::Direction,
    ) -> impl Iterator<Item = vf2::NodeIndex> {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                match direction {
                    vf2::Direction::Outgoing => Direction::Outgoing,
                    vf2::Direction::Incoming => Direction::Incoming,
                },
            )
            .map(|neighbor| neighbor.index())
    }

    fn contains_edge(&self, source: vf2::NodeIndex, target: vf2::NodeIndex) -> bool {
        self.0.contains_edge(
            petgraph::graph::NodeIndex::new(source),
            petgraph::graph::NodeIndex::new(target),
        )
    }

    fn edge_label(
        &self,
        source: vf2::NodeIndex,
        target: vf2::NodeIndex,
    ) -> Option<&Self::EdgeLabel> {
        self.0
            .find_edge(
                petgraph::graph::NodeIndex::new(source),
                petgraph::graph::NodeIndex::new(target),
            )
            .and_then(|edge| self.0.edge_weight(edge))
    }
}

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

#[test]
fn vertex_cascade_creation() {
    let (mut replica_a, mut replica_b) = twins_log::<BehaviortreeLog>();

    let a1 = replica_a
        .send(Behaviortree::Root(Root::Main(BehaviorTree::Child(
            Box::new(TreeNode::ExecutionNode(ExecutionNode::Action(
                Action::OpenDoor(OpenDoor::ActionFeat(ActionFeat::ExecutionNodeFeat(
                    ExecutionNodeFeat::Inflowports(NestedList::Insert {
                        pos: 0,
                        value: InFlowPort::New,
                    }),
                ))),
            ))),
        ))))
        .unwrap();

    // println!("{:#?}", replica_a.query(Read::new()).root);
    // println!(
    //     "{:#?}",
    //     replica_a
    //         .query(Read::new())
    //         .refs
    //         .node_weights()
    //         .map(|n| match n {
    //             Instance::BlackboardEntryId(id) => format!("BlackboardEntryId({})", id.0),
    //             Instance::OutFlowPortId(id) => format!("OutFlowPortId({})", id.0),
    //             Instance::InFlowPortId(id) => format!("InFlowPortId({})", id.0),
    //         })
    //         .collect::<Vec<_>>()
    //         .join(",")
    // );
    assert_eq!(replica_a.query(Read::new()).refs.node_count(), 1);
}

/// This test checks that when we update a blackboard entry while another replica deletes it concurrently,
/// the delete do reset the blackboard entry to the default value, but the update is then applied.
/// In addition, we check that the vertex is revived in the reference graph.
#[test]
fn blackboard_update_delete() {
    let (mut replica_a, mut replica_b) = twins_log::<BehaviortreeLog>();

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

    let run = RunConfig::new(0.4, 4, 1_000, None, None, true, false);
    let runs = vec![run.clone(); 1];

    let config = FuzzerConfig::<BehaviortreeLog>::new(
        "bt",
        runs,
        true,
        |a, b| {
            let package = a.root == b.root;
            if !package {
                println!("Package mismatch");
                println!("----- Root A -----");
                println!("{:#?}", a.root);
                println!("----- Root B -----");
                println!("{:#?}", b.root);
                return false;
            }

            if a.refs.node_count() == 0 && b.refs.node_count() == 0 {
                // If both graphs are empty, skip the isomorphism check to avoid false negatives due to different node IDs.
                return true;
            } else {
                let refs = vf2::isomorphisms(&Vf2GraphView(&a.refs), &Vf2GraphView(&b.refs))
                    .default_eq()
                    .first()
                    .is_some();
                if !refs {
                    println!(
                        "Graph isomorphism mismatch: nodes {} vs {}, edges {} vs {}",
                        a.refs.node_count(),
                        b.refs.node_count(),
                        a.refs.edge_count(),
                        b.refs.edge_count()
                    );
                    println!("----- Graph A -----");
                    println!("{:#?}", a.refs);
                    println!("----- Graph B -----");
                    println!("{:#?}", b.refs);
                }
                return refs;
            }
        },
        false,
    );

    fuzzer::<BehaviortreeLog>(config);
}
