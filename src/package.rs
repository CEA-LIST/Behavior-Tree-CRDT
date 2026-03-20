/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __package {
    pub use moirai_protocol::crdt::query::Read;
    pub use moirai_protocol::crdt::eval::EvalNested;
    pub use moirai_protocol::state::log::IsLog;
    pub use moirai_protocol::clock::version_vector::Version;
    pub use moirai_protocol::event::Event;
    pub use moirai_protocol::crdt::query::QueryOperation;
    pub use moirai_protocol::state::sink::ObjectPath;
    pub use moirai_protocol::state::sink::SinkEffect;
    pub use crate::classifiers::*;
    pub use moirai_crdt::policy::LwwPolicy;
    pub use moirai_protocol::state::po_log::VecLog;
    pub use moirai_protocol::crdt::pure_crdt::PureCRDT;
    pub use moirai_protocol::state::sink::SinkCollector;
    pub use moirai_protocol::state::sink::IsLogSink;
    pub use crate::references::*;
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
    pub refs: <__package::ReferenceManager<
        __package::LwwPolicy,
    > as __package::PureCRDT>::Value,
}
#[derive(Debug, Clone, Default)]
pub struct BehaviortreeLog {
    root_log: __package::RootLog,
    reference_manager_log: __package::VecLog<
        __package::ReferenceManager<__package::LwwPolicy>,
    >,
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
            Behaviortree::AddReference(o) => {
                self.reference_manager_log
                    .is_enabled(&__package::ReferenceManager::AddArc(o.clone()))
            }
            Behaviortree::RemoveReference(o) => {
                self.reference_manager_log
                    .is_enabled(&__package::ReferenceManager::RemoveArc(o.clone()))
            }
        }
    }
    fn effect(&mut self, event: __package::Event<Self::Op>) {
        let mut sink = __package::SinkCollector::new();
        match event.op().clone() {
            Behaviortree::Root(o) => {
                __package::IsLogSink::effect_with_sink(
                    &mut self.root_log,
                    __package::Event::unfold(event.clone(), o),
                    __package::ObjectPath::new("behaviortree").field("root"),
                    &mut sink,
                )
            }
            Behaviortree::AddReference(o) => {
                self.reference_manager_log
                    .effect(
                        __package::Event::unfold(
                            event.clone(),
                            __package::ReferenceManager::AddArc(o),
                        ),
                    )
            }
            Behaviortree::RemoveReference(o) => {
                self.reference_manager_log
                    .effect(
                        __package::Event::unfold(
                            event.clone(),
                            __package::ReferenceManager::RemoveArc(o),
                        ),
                    )
            }
        }
        for sink in sink.into_sinks() {
            match sink.effect() {
                __package::SinkEffect::Create | __package::SinkEffect::Update => {
                    let vertex_ops = __package::instance_from_path(&sink.path())
                        .map(|instance| __package::ReferenceManager::AddVertex {
                            id: instance,
                        });
                    if let Some(o) = vertex_ops {
                        self.reference_manager_log
                            .effect(__package::Event::unfold(event.clone(), o));
                    }
                }
                __package::SinkEffect::Delete => {
                    let graph = self.reference_manager_log.eval(__package::Read::new());
                    let removals = graph
                        .node_weights()
                        .filter(|n| {
                            sink.path().is_prefix_of(__package::instance_path(n))
                        })
                        .collect::<Vec<_>>();
                    for removal in removals {
                        let removal_event = __package::Event::unfold(
                            event.clone(),
                            __package::ReferenceManager::RemoveVertex {
                                id: removal.clone(),
                            },
                        );
                        self.reference_manager_log.effect(removal_event);
                    }
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
        self.reference_manager_log.redundant_by_parent(version, conservative);
    }
    fn is_default(&self) -> bool {
        true && self.root_log.is_default()
    }
}
impl __package::EvalNested<__package::Read<<Self as __package::IsLog>::Value>>
for BehaviortreeLog {
    fn execute_query(
        &self,
        _q: __package::Read<<Self as __package::IsLog>::Value>,
    ) -> <__package::Read<
        <Self as __package::IsLog>::Value,
    > as __package::QueryOperation>::Response {
        BehaviortreeValue {
            root: self.root_log.execute_query(__package::Read::new()),
            refs: self.reference_manager_log.execute_query(__package::Read::new()),
        }
    }
}
