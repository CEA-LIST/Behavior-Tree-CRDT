use moirai_protocol::{
    crdt::policy::Policy,
    state::sink::PathSegment::{Field, ListElement, MapEntry, Variant},
    state::sink::{ObjectPath, Sink, SinkEffect},
};

mod __references {
    pub use moirai_macros::typed_graph;
}

fn instance_from_path(path: &ObjectPath) -> Option<Instance> {
    let segs = path.segments();

    match segs {
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Action"),
            Variant("OpenDoor"),
            Field("action_feat"),
            Field("execution_node_feat"),
            Field("outflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Action"),
            Variant("OpenDoor"),
            Field("action_feat"),
            Field("execution_node_feat"),
            Field("inflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Action"),
            Variant("EnterRoom"),
            Field("action_feat"),
            Field("execution_node_feat"),
            Field("outflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Action"),
            Variant("EnterRoom"),
            Field("action_feat"),
            Field("execution_node_feat"),
            Field("inflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Action"),
            Variant("CloseDoor"),
            Field("action_feat"),
            Field("execution_node_feat"),
            Field("outflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Action"),
            Variant("CloseDoor"),
            Field("action_feat"),
            Field("execution_node_feat"),
            Field("inflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Condition"),
            Variant("IsDoorOpen"),
            Field("condition_feat"),
            Field("execution_node_feat"),
            Field("outflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("child"),
            Variant("ExecutionNode"),
            Variant("Condition"),
            Variant("IsDoorOpen"),
            Field("condition_feat"),
            Field("execution_node_feat"),
            Field("inflowports"),
            ListElement(_),
        ] => Some(Instance::DataFlowPortId(DataFlowPortId(path.clone()))),
        [
            ..,
            Field("main"),
            Field("blackboard"),
            Field("entries"),
            ListElement(_),
        ] => Some(Instance::BlackboardEntryId(BlackboardEntryId(path.clone()))),
        _ => None,
    }
}

pub fn vertex_ops_from_sink<P: Policy>(sink: &Sink) -> Option<ReferenceManager<P>> {
    let instance = instance_from_path(sink.path())?;

    match sink.effect() {
        SinkEffect::Create | SinkEffect::Update => {
            Some(ReferenceManager::AddVertex { id: instance })
        }
        SinkEffect::Delete => Some(ReferenceManager::RemoveVertex { id: instance }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataFlowPortId(pub ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlackboardEntryId(pub ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataFlowPortEntryEdge;
__references::typed_graph! {
    graph : ReferenceManager, vertex : Instance, edge : Ref, arcs_type : Refs, vertices {
    DataFlowPortId, BlackboardEntryId }, connections { DataFlowPortEntry : DataFlowPortId
    -> BlackboardEntryId(DataFlowPortEntryEdge) [0, 1] }
}
