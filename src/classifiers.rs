/// Auto-generated code by 🅰🆁🅰🅲🅷🅽🅴 - do not edit directly
mod __classifiers {
    pub use moirai_crdt::list::eg_walker::List;
    pub use moirai_crdt::list::nested_list::NestedListLog;
    pub use moirai_crdt::option::OptionLog;
    pub use moirai_macros::record;
    pub use moirai_macros::union;
    pub use moirai_protocol::state::event_graph::EventGraph;
}
__classifiers::record!(
    Root { behaviortrees : __classifiers::NestedListLog < BehaviorTreeLog >, main :
    BehaviorTreeLog, }
);
__classifiers::record!(
    BehaviorTree { id : __classifiers::EventGraph < __classifiers::List < char > >, child
    : Box < TreeNodeLog >, blackboard : BlackboardLog, }
);
__classifiers::union!(
    TreeNode = ExecutionNode(ExecutionNode, ExecutionNodeLog)
        | Decorator(Decorator, DecoratorLog)
        | ControlNode(ControlNode, ControlNodeLog)
        | SubTree(SubTree, SubTreeLog)
);
__classifiers::record!(
    TreeNodeFeat { id : __classifiers::EventGraph < __classifiers::List < char > >, name
    : __classifiers::OptionLog < __classifiers::EventGraph < __classifiers::List < char >
    >>, }
);
__classifiers::record!(
    Blackboard { entries : __classifiers::NestedListLog < BlackboardEntryLog >, }
);
__classifiers::union!(
    ExecutionNode = Action(Action, ActionLog) | Condition(Condition, ConditionLog)
);
__classifiers::record!(
    ExecutionNodeFeat { tree_node_feat : TreeNodeFeatLog, outflowports :
    __classifiers::NestedListLog < OutFlowPortLog >, inflowports :
    __classifiers::NestedListLog < InFlowPortLog >, }
);
__classifiers::union!(
    DataFlowPort = OutFlowPort(OutFlowPort, OutFlowPortLog) | InFlowPort(InFlowPort, InFlowPortLog)
);
__classifiers::record!(DataFlowPortFeat {});
__classifiers::record!(OutFlowPort {
    data_flow_port_feat: DataFlowPortFeatLog,
});
__classifiers::record!(InFlowPort {
    data_flow_port_feat: DataFlowPortFeatLog,
});
__classifiers::union!(Decorator = Inverter(Inverter, InverterLog));
__classifiers::record!(
    DecoratorFeat { tree_node_feat : TreeNodeFeatLog, child : Box < TreeNodeLog >, }
);
__classifiers::union!(
    ControlNode = Sequence(Sequence, SequenceLog) | Fallback(Fallback, FallbackLog)
);
__classifiers::record!(
    ControlNodeFeat { tree_node_feat : TreeNodeFeatLog, children :
    __classifiers::NestedListLog < Box < TreeNodeLog > >, }
);
__classifiers::record!(Sequence {
    control_node_feat: ControlNodeFeatLog,
});
__classifiers::record!(Fallback {
    control_node_feat: ControlNodeFeatLog,
});
__classifiers::union!(
    Action = OpenDoor(OpenDoor, OpenDoorLog)
        | EnterRoom(EnterRoom, EnterRoomLog)
        | CloseDoor(CloseDoor, CloseDoorLog)
);
__classifiers::record!(ActionFeat {
    execution_node_feat: ExecutionNodeFeatLog,
});
__classifiers::union!(Condition = IsDoorOpen(IsDoorOpen, IsDoorOpenLog));
__classifiers::record!(ConditionFeat {
    execution_node_feat: ExecutionNodeFeatLog,
});
__classifiers::record!(
    BlackboardEntry { key : __classifiers::EventGraph < __classifiers::List < char > >,
    value : __classifiers::EventGraph < __classifiers::List < char > >, }
);
__classifiers::record!(Inverter {
    decorator_feat: DecoratorFeatLog,
});
__classifiers::record!(IsDoorOpen {
    condition_feat: ConditionFeatLog,
});
__classifiers::record!(OpenDoor {
    action_feat: ActionFeatLog,
});
__classifiers::record!(EnterRoom {
    action_feat: ActionFeatLog,
});
__classifiers::record!(CloseDoor {
    action_feat: ActionFeatLog,
});
__classifiers::record!(SubTree {
    tree_node_feat: TreeNodeFeatLog,
    tree: BehaviorTreeLog,
});
