/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __references {
    pub use moirai_macros::typed_graph;
    pub use moirai_protocol::state::sink::ObjectPath;
    pub use moirai_protocol::state::sink::PathSegment::{
        Field, ListElement, MapEntry, Variant,
    };
}
pub fn instance_from_path(path: &__references::ObjectPath) -> Option<Instance> {
    let segs = path.segments();
    match segs {
        [.., __references::Field("outflowports"), __references::ListElement(_)] => {
            Some(Instance::OutFlowPortId(OutFlowPortId(path.clone())))
        }
        [.., __references::Field("inflowports"), __references::ListElement(_)] => {
            Some(Instance::InFlowPortId(InFlowPortId(path.clone())))
        }
        [.., __references::Field("entries"), __references::ListElement(_)] => {
            Some(Instance::BlackboardEntryId(BlackboardEntryId(path.clone())))
        }
        _ => None,
    }
}
pub fn instance_path(instance: &Instance) -> &__references::ObjectPath {
    match instance {
        Instance::OutFlowPortId(id) => &id.0,
        Instance::InFlowPortId(id) => &id.0,
        Instance::BlackboardEntryId(id) => &id.0,
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutFlowPortId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InFlowPortId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlackboardEntryId(pub __references::ObjectPath);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutFlowPortEntryEdge;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InFlowPortEntryEdge;
__references::typed_graph! {
    graph : ReferenceManager, vertex : Instance, edge : Ref, arcs_type : Refs, vertices {
    OutFlowPortId, InFlowPortId, BlackboardEntryId }, connections {
    OutFlowPortToBlackboardEntry : OutFlowPortId ->
    BlackboardEntryId(OutFlowPortEntryEdge) [0, 1], InFlowPortToBlackboardEntry :
    InFlowPortId -> BlackboardEntryId(InFlowPortEntryEdge) [0, 1] }
}
