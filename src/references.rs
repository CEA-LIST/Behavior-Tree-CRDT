use moirai_protocol::{
    crdt::policy::Policy,
    state::sink::{ObjectPath, Sink, SinkEffect},
};

mod __references {
    pub use moirai_macros::typed_graph;
}

fn instance_from_path(path: &ObjectPath) -> Option<Instance> {
    use moirai_protocol::state::sink::PathSegment::*;

    let segs = path.segments();

    match segs {
        [.., Field("blackboard"), Field("entries"), ListElement(_)] => {
            Some(Instance::BlackboardEntryId(BlackboardEntryId(path.clone())))
        }
        [.., Field("outflowports"), ListElement(_)] => {
            Some(Instance::DataFlowPortId(DataFlowPortId(path.clone())))
        }
        [.., Field("inflowports"), ListElement(_)] => {
            Some(Instance::DataFlowPortId(DataFlowPortId(path.clone())))
        }
        _ => None,
    }
}

pub fn vertex_ops_from_sink<P: Policy>(sink: &Sink) -> Option<ReferenceManager<P>> {
    let Some(instance) = instance_from_path(sink.object_path()) else {
        return None;
    };

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
