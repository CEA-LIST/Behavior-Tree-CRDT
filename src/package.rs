/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use crate::classifiers::*;
    pub use crate::references::*;
    pub use moirai_crdt::policy::FairPolicy;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::event::Event;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::state::object_path::ObjectPath;
    pub use moirai_protocol::state::po_log::POLog;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_protocol::state::sink::SinkCollector;
    pub use moirai_protocol::state::sink::SinkEffect;
    pub use moirai_protocol::state::sink::SinkOwnership;
    pub use moirai_protocol::utils::intern_str::InternalizeOp;
    pub use moirai_protocol::utils::intern_str::Interner;
}
pub type ReferenceManagerLog = __package::POLog<
    __package::ReferenceManager<__package::FairPolicy>,
    __package::ReferenceManagerState<__package::FairPolicy>,
>;
#[derive(Debug, Clone)]
pub enum Behaviortree {
    Root(__package::Root),
    AddReference(__package::Refs),
    RemoveReference(__package::Refs),
}
#[derive(Debug, Clone, Default)]
pub struct BehaviortreeValue {
    pub root: __package::RootValue,
    pub refs: <__package::ReferenceManager<__package::FairPolicy> as __package::PureCRDT>::Value,
}
#[derive(Debug, Clone, Default)]
pub struct BehaviortreeLog {
    root_log: __package::RootLog,
    reference_manager_log: __package::VecLog<__package::ReferenceManager<__package::FairPolicy>>,
}
impl BehaviortreeLog {
    pub fn root_log(&self) -> &__package::RootLog {
        &self.root_log
    }
    pub fn reference_manager_log(
        &self,
    ) -> &__package::VecLog<__package::ReferenceManager<__package::FairPolicy>> {
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
    fn effect(
        &mut self,
        event: __package::Event<Self::Op>,
        _path: __package::ObjectPath,
        _sink: &mut __package::SinkCollector,
        _ownership: __package::SinkOwnership,
    ) {
        let mut sink = __package::SinkCollector::new();
        match event.op().clone() {
            Behaviortree::Root(o) => __package::IsLog::effect(
                &mut self.root_log,
                __package::Event::unfold(event.clone(), o),
                __package::ObjectPath::new("behaviortree").field("root"),
                &mut sink,
                __package::SinkOwnership::Owned,
            ),
            Behaviortree::AddReference(o) => self.reference_manager_log.effect(
                __package::Event::unfold(event.clone(), __package::ReferenceManager::AddArc(o)),
                __package::ObjectPath::new("behaviortree"),
                &mut __package::SinkCollector::new(),
                __package::SinkOwnership::Owned,
            ),
            Behaviortree::RemoveReference(o) => self.reference_manager_log.effect(
                __package::Event::unfold(event.clone(), __package::ReferenceManager::RemoveArc(o)),
                __package::ObjectPath::new("behaviortree"),
                &mut __package::SinkCollector::new(),
                __package::SinkOwnership::Owned,
            ),
        }
        for sink in sink.into_sinks() {
            match sink.effect() {
                __package::SinkEffect::Create | __package::SinkEffect::Update => {
                    let vertex_ops = __package::instance_from_path(sink.path())
                        .map(|instance| __package::ReferenceManager::AddVertex { id: instance });
                    if let Some(o) = vertex_ops {
                        self.reference_manager_log.effect(
                            __package::Event::unfold(event.clone(), o),
                            __package::ObjectPath::new("behaviortree"),
                            &mut __package::SinkCollector::new(),
                            __package::SinkOwnership::Owned,
                        );
                    }
                }
                __package::SinkEffect::Delete => {
                    self.reference_manager_log.effect(
                        __package::Event::unfold(
                            event.clone(),
                            __package::ReferenceManager::DeleteSubtree {
                                prefix: sink.path().clone(),
                            },
                        ),
                        __package::ObjectPath::new("behaviortree"),
                        &mut __package::SinkCollector::new(),
                        __package::SinkOwnership::Owned,
                    );
                }
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
impl __package::InternalizeOp for Behaviortree {
    fn internalize(self, interner: &__package::Interner) -> Self {
        match self {
            Behaviortree::Root(op) => Behaviortree::Root(op.clone()),
            Behaviortree::AddReference(op) => Behaviortree::AddReference(op.internalize(interner)),
            Behaviortree::RemoveReference(op) => {
                Behaviortree::RemoveReference(op.internalize(interner))
            }
        }
    }
}
