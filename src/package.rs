use moirai_protocol::state::sink::{IsLogSink, ObjectPath};

use crate::references::vertex_ops_from_sink;

mod __package {
    pub use crate::classifiers::*;
    pub use crate::references::*;

    pub use moirai_crdt::policy::LwwPolicy;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::event::Event;

    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::state::po_log::VecLog;
}
#[derive(Debug, Clone)]
pub enum Behaviortree {
    Root(__package::Root),
    AddReference(__package::Refs),
    RemoveReference(__package::Refs),
}
#[derive(Debug, Clone, Default)]
pub struct BehaviortreeValue {
    pub root: __package::RootValue,
    pub refs: <__package::ReferenceManager<__package::LwwPolicy> as __package::PureCRDT>::Value,
}
#[derive(Debug, Clone, Default)]
pub struct BehaviortreeLog {
    root_log: __package::RootLog,
    // TODO: must use a VecLog!
    reference_manager_log: __package::VecLog<__package::ReferenceManager<__package::LwwPolicy>>,
}
impl BehaviortreeLog {
    pub fn root_log(&self) -> &__package::RootLog {
        &self.root_log
    }
    pub fn reference_manager_log(
        &self,
    ) -> &__package::VecLog<__package::ReferenceManager<__package::LwwPolicy>> {
        &self.reference_manager_log
    }
}
impl __package::IsLog for BehaviortreeLog {
    type Value = BehaviortreeValue;
    type Op = Behaviortree;
    fn is_enabled(&self, op: &Self::Op) -> bool {
        match op {
            Behaviortree::Root(o) => self.root_log.is_enabled(o),
            Behaviortree::AddReference(o) => self
                .reference_manager_log
                .is_enabled(&__package::ReferenceManager::AddArc(o.clone())),
            Behaviortree::RemoveReference(o) => self
                .reference_manager_log
                .is_enabled(&__package::ReferenceManager::RemoveArc(o.clone())),
        }
    }
    fn effect(&mut self, event: __package::Event<Self::Op>) {
        let mut sink = moirai_protocol::state::sink::SinkCollector::new();
        let path = ObjectPath::new("root");
        match event.op().clone() {
            Behaviortree::Root(o) => {
                self.root_log.effect_with_sink(
                    __package::Event::unfold(event.clone(), o),
                    path,
                    &mut sink,
                );
            }
            Behaviortree::AddReference(refs) => self.reference_manager_log.effect(
                __package::Event::unfold(event.clone(), __package::ReferenceManager::AddArc(refs)),
            ),
            Behaviortree::RemoveReference(refs) => {
                self.reference_manager_log.effect(__package::Event::unfold(
                    event.clone(),
                    __package::ReferenceManager::RemoveArc(refs),
                ))
            }
        }
        for sink in sink.into_sinks() {
            // TODO: event id may not be uniques in the Typed Graph!
            if let Some(op) = vertex_ops_from_sink::<__package::LwwPolicy>(&sink) {
                let vertex_event = __package::Event::unfold(event.clone(), op);
                self.reference_manager_log.effect(vertex_event);
            }
        }
    }
    fn stabilize(&mut self, version: &__package::Version) {
        self.root_log.stabilize(version);
        self.reference_manager_log.stabilize(version);
    }
    fn redundant_by_parent(&mut self, version: &__package::Version, conservative: bool) {
        self.root_log.redundant_by_parent(version, conservative);
        self.reference_manager_log
            .redundant_by_parent(version, conservative);
    }
    fn is_default(&self) -> bool {
        true && self.root_log.is_default()
    }
}
impl __package::EvalNested<__package::Read<<Self as __package::IsLog>::Value>> for BehaviortreeLog {
    fn execute_query(
        &self,
        _q: __package::Read<<Self as __package::IsLog>::Value>,
    ) -> <__package::Read<<Self as __package::IsLog>::Value> as __package::QueryOperation>::Response
    {
        BehaviortreeValue {
            root: self.root_log.execute_query(__package::Read::new()),
            refs: self
                .reference_manager_log
                .execute_query(__package::Read::new()),
        }
    }
}
