use behaviortree::{
    classifiers::{BehaviorTree, Blackboard, BlackboardEntry, Root},
    package::{Behaviortree, BehaviortreeLog},
    references::{
        BlackboardEntryId, DataFlowPortEntryEdge, DataFlowPortId, Instance, Ref, ReferenceManager,
        Refs,
    },
};
use moirai_crdt::list::{eg_walker::List, nested_list::NestedList};
use moirai_crdt::policy::LwwPolicy;
use moirai_macros::typed_graph::Arc;
use moirai_protocol::{
    broadcast::{message::EventMessage, tcsb::Tcsb},
    crdt::query::Read,
    replica::{IsReplica, Replica},
    state::{po_log::VecLog, sink::ObjectPath},
};
use petgraph::visit::EdgeRef;

type RefReplica = Replica<VecLog<ReferenceManager<LwwPolicy>>, Tcsb<ReferenceManager<LwwPolicy>>>;

fn graph_vertices(replica: &Replica<BehaviortreeLog, Tcsb<Behaviortree>>) -> Vec<String> {
    let mut vertices = replica
        .query(Read::new())
        .refs
        .node_weights()
        .map(|i| match i {
            Instance::DataFlowPortId(data_flow_port_id) => {
                format!("DataFlowPortId({})", data_flow_port_id.0)
            }
            Instance::BlackboardEntryId(blackboard_entry_id) => {
                format!("BlackboardEntryId({})", blackboard_entry_id.0)
            }
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
            Ref::DataFlowPortEntry(_) => "DataFlowPortEntry".to_string(),
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
}

fn ref_graph_vertices(replica: &RefReplica) -> Vec<String> {
    let mut vertices = replica
        .query(Read::new())
        .node_weights()
        .map(instance_name)
        .collect::<Vec<_>>();
    vertices.sort();
    vertices
}

fn ref_graph_edges(replica: &RefReplica) -> Vec<String> {
    let graph = replica.query(Read::new());
    let mut edges = graph
        .edge_references()
        .map(|edge| {
            let source = instance_name(graph.node_weight(edge.source()).unwrap());
            let target = instance_name(graph.node_weight(edge.target()).unwrap());
            let kind = match edge.weight() {
                Ref::DataFlowPortEntry(_) => "DataFlowPortEntry",
            };
            format!("{source} -> {target} ({kind})")
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
}

fn instance_name(instance: &Instance) -> String {
    match instance {
        Instance::DataFlowPortId(data_flow_port_id) => {
            format!("DataFlowPortId({})", data_flow_port_id.0)
        }
        Instance::BlackboardEntryId(blackboard_entry_id) => {
            format!("BlackboardEntryId({})", blackboard_entry_id.0)
        }
    }
}

fn ref_twins() -> (RefReplica, RefReplica) {
    (
        Replica::<VecLog<ReferenceManager<LwwPolicy>>, Tcsb<ReferenceManager<LwwPolicy>>>::new(
            "a".to_string(),
        ),
        Replica::<VecLog<ReferenceManager<LwwPolicy>>, Tcsb<ReferenceManager<LwwPolicy>>>::new(
            "b".to_string(),
        ),
    )
}

fn blackboard_entry_id(name: &str) -> BlackboardEntryId {
    BlackboardEntryId(
        ObjectPath::new("root")
            .field("main")
            .field("blackboard")
            .field("entries")
            .map_entry(name.to_string()),
    )
}

fn data_flow_port_id(name: &str) -> DataFlowPortId {
    DataFlowPortId(
        ObjectPath::new("root")
            .field("main")
            .field("child")
            .field("ports")
            .map_entry(name.to_string()),
    )
}

fn port_to_entry_ref(port: &str, entry: &str) -> Refs {
    Refs::DataFlowPortEntry(Arc {
        source: data_flow_port_id(port),
        target: blackboard_entry_id(entry),
        kind: DataFlowPortEntryEdge,
    })
}

fn add_vertex(replica: &mut RefReplica, id: Instance) -> EventMessage<ReferenceManager<LwwPolicy>> {
    replica
        .send(ReferenceManager::AddVertex { id })
        .expect("AddVertex should be enabled")
}

fn remove_vertex(
    replica: &mut RefReplica,
    id: Instance,
) -> EventMessage<ReferenceManager<LwwPolicy>> {
    replica
        .send(ReferenceManager::RemoveVertex { id })
        .expect("RemoveVertex should be enabled")
}

fn add_arc(replica: &mut RefReplica, arc: Refs) -> EventMessage<ReferenceManager<LwwPolicy>> {
    replica
        .send(ReferenceManager::AddArc(arc))
        .expect("AddArc should be enabled")
}

fn assert_ref_convergence(replica_a: &RefReplica, replica_b: &RefReplica) {
    assert_eq!(ref_graph_vertices(replica_a), ref_graph_vertices(replica_b));
    assert_eq!(ref_graph_edges(replica_a), ref_graph_edges(replica_b));
}

#[test]
fn creating_blackboard_entry_adds_vertex_to_reference_graph() {
    let mut replica = Replica::<BehaviortreeLog, Tcsb<Behaviortree>>::new("a".to_string());

    replica
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

    let vertices = graph_vertices(&replica);
    assert_eq!(vertices.len(), 1);
    assert!(vertices[0].starts_with("BlackboardEntryId(root/main/blackboard/entries/"));
}

#[test]
fn typed_graph_add_arc_between_existing_vertices() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, port.clone());
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_b, entry.clone());
    replica_a.receive(e2);

    let e3 = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));
    replica_b.receive(e3);

    assert_eq!(replica_a.query(Read::new()).node_count(), 2);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 1);
    assert_eq!(
        ref_graph_edges(&replica_a),
        vec![format!(
            "{} -> {} (DataFlowPortEntry)",
            instance_name(&port),
            instance_name(&entry)
        )]
    );
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_concurrent_add_same_vertex_is_idempotent() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));

    let event_a = add_vertex(&mut replica_a, port.clone());
    let event_b = add_vertex(&mut replica_b, port);
    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).node_count(), 1);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_concurrent_add_same_arc_is_idempotent() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, port);
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, entry);
    replica_b.receive(e2);

    let event_a = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));
    let event_b = add_arc(&mut replica_b, port_to_entry_ref("port-1", "entry-1"));
    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).edge_count(), 1);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_remove_vertex_cascades_existing_arc() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, port);
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, entry.clone());
    replica_b.receive(e2);
    let e3 = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));
    replica_b.receive(e3);

    let e4 = remove_vertex(&mut replica_b, entry);
    replica_a.receive(e4);

    assert_eq!(replica_a.query(Read::new()).node_count(), 1);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 0);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_concurrent_add_vertex_and_remove_vertex_keeps_vertex() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let init = add_vertex(&mut replica_a, entry.clone());
    replica_b.receive(init);

    let event_a = add_vertex(&mut replica_a, entry.clone());
    let event_b = remove_vertex(&mut replica_b, entry);
    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).node_count(), 1);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 0);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_concurrent_add_vertex_and_remove_vertex_drops_existing_arc_but_keeps_vertex() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, port.clone());
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, entry.clone());
    replica_b.receive(e2);
    let e3 = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));
    replica_b.receive(e3);

    let event_a = add_vertex(&mut replica_a, entry.clone());
    let event_b = remove_vertex(&mut replica_b, entry);
    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).node_count(), 2);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 0);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_concurrent_remove_vertex_and_add_arc_hides_arc() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, port);
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, entry.clone());
    replica_b.receive(e2);

    let event_a = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));
    let event_b = remove_vertex(&mut replica_b, entry);
    replica_a.receive(event_b);
    replica_b.receive(event_a);

    assert_eq!(replica_a.query(Read::new()).node_count(), 1);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 0);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_reinserting_vertex_revives_hidden_concurrent_arc() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, port);
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, entry.clone());
    replica_b.receive(e2);

    let event_a = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));
    let event_b = remove_vertex(&mut replica_b, entry.clone());
    replica_a.receive(event_b);
    replica_b.receive(event_a);

    let e3 = add_vertex(&mut replica_a, entry);
    replica_b.receive(e3);

    assert_eq!(replica_a.query(Read::new()).node_count(), 2);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 1);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_reinserting_same_vertex_after_delete_does_not_duplicate_it() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let entry = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));

    let e1 = add_vertex(&mut replica_a, entry.clone());
    replica_b.receive(e1);
    let e2 = remove_vertex(&mut replica_a, entry.clone());
    replica_b.receive(e2);
    let e3 = add_vertex(&mut replica_b, entry);
    replica_a.receive(e3);

    assert_eq!(replica_a.query(Read::new()).node_count(), 1);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 0);
    assert_ref_convergence(&replica_a, &replica_b);
}

#[test]
fn typed_graph_mixed_add_wins_trace_converges() {
    let (mut replica_a, mut replica_b) = ref_twins();
    let port = Instance::DataFlowPortId(data_flow_port_id("port-1"));
    let entry_a = Instance::BlackboardEntryId(blackboard_entry_id("entry-1"));
    let entry_b = Instance::BlackboardEntryId(blackboard_entry_id("entry-2"));

    let e1 = add_vertex(&mut replica_a, port.clone());
    replica_b.receive(e1);
    let e2 = add_vertex(&mut replica_a, entry_a.clone());
    replica_b.receive(e2);
    let e3 = add_arc(&mut replica_a, port_to_entry_ref("port-1", "entry-1"));

    let e4 = add_vertex(&mut replica_b, entry_b.clone());
    let e5 = add_arc(&mut replica_b, port_to_entry_ref("port-1", "entry-2"));
    let e6 = remove_vertex(&mut replica_b, entry_b);

    replica_b.receive(e3);
    replica_a.receive(e4);
    replica_a.receive(e5);
    replica_a.receive(e6);

    assert_eq!(replica_a.query(Read::new()).node_count(), 2);
    assert_eq!(replica_a.query(Read::new()).edge_count(), 1);
    assert_eq!(ref_graph_edges(&replica_a).len(), 1);
    assert_ref_convergence(&replica_a, &replica_b);
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

    println!("{:?}", replica_a.query(Read::new()).root);

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
    let runs = vec![run.clone(); 100];

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
