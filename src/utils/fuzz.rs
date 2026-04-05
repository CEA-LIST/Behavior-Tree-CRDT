use moirai_crdt::list::nested_list::{NestedList, NestedListLog};
use moirai_fuzz::{
    metrics::{FuzzMetrics, StructureMetrics},
    op_generator::OpGeneratorNested,
};
use moirai_protocol::{
    crdt::{eval::EvalNested, query::Read},
    state::log::IsLog,
};
use rand::{
    Rng, RngExt,
    distr::{Distribution, weighted::WeightedIndex},
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{
    classifiers::*,
    package::{Behaviortree, BehaviortreeLog},
    references::compute_arc_constraints,
};

fn generate_boxed_tree_node(log: &TreeNodeKindLog, rng: &mut impl Rng) -> Box<TreeNodeKind> {
    Box::new(log.generate(rng))
}

fn generate_boxed_tree_list(
    log: &NestedListLog<Box<TreeNodeKindLog>>,
    rng: &mut impl Rng,
) -> NestedList<Box<TreeNodeKind>> {
    enum Choice {
        Insert,
        Update,
        Delete,
    }

    let positions = log.positions().execute_query(Read::new());
    let choice = if positions.is_empty() {
        Choice::Insert
    } else {
        [Choice::Insert, Choice::Update, Choice::Delete]
            .into_iter()
            .choose(rng)
            .unwrap()
    };

    let op = match choice {
        Choice::Insert => {
            let pos = rng.random_range(0..=positions.len());
            let op = generate_boxed_tree_node(&Box::<TreeNodeKindLog>::default(), rng);
            NestedList::Insert { pos, op }
        }
        Choice::Update => {
            let pos = rng.random_range(0..positions.len());
            let target_id = &positions[pos];
            let op = log
                .children()
                .get_child(target_id)
                .map(|child| generate_boxed_tree_node(child, rng))
                .unwrap_or_else(|| {
                    generate_boxed_tree_node(&Box::<TreeNodeKindLog>::default(), rng)
                });
            NestedList::Update { pos, op }
        }
        Choice::Delete => {
            let pos = rng.random_range(0..positions.len());
            NestedList::Delete { pos }
        }
    };

    assert!(log.is_enabled(&op));
    op
}

impl OpGeneratorNested for BehaviortreeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Root,
            AddReference,
            RemoveReference,
        }

        let refs = self.reference_manager_log().execute_query(Read::new());
        let constraints = compute_arc_constraints(&refs);

        let choice = if refs.node_count() < 2
            || (constraints.addable.is_empty() && constraints.removable.is_empty())
        {
            Choice::Root
        } else if constraints.removable.is_empty() {
            [Choice::Root, Choice::AddReference][WeightedIndex::new([8, 3]).unwrap().sample(rng)]
        } else if constraints.addable.is_empty() {
            [Choice::Root, Choice::RemoveReference][WeightedIndex::new([8, 2]).unwrap().sample(rng)]
        } else {
            [Choice::Root, Choice::AddReference, Choice::RemoveReference]
                [WeightedIndex::new([8, 3, 2]).unwrap().sample(rng)]
        };

        match choice {
            Choice::Root => Behaviortree::Root(self.root_log().generate(rng)),
            Choice::AddReference => Behaviortree::AddReference(
                constraints
                    .addable
                    .choose(rng)
                    .expect("addable references should not be empty")
                    .clone(),
            ),
            Choice::RemoveReference => Behaviortree::RemoveReference(
                constraints
                    .removable
                    .choose(rng)
                    .expect("removable references should not be empty")
                    .clone(),
            ),
        }
    }
}

impl OpGeneratorNested for RootLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Behaviortrees,
            Main,
        }

        match [Choice::Behaviortrees, Choice::Main]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::Behaviortrees => Root::Behaviortrees(self.behaviortrees().generate(rng)),
            Choice::Main => Root::Main(self.main().generate(rng)),
        }
    }
}

impl OpGeneratorNested for BehaviorTreeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Id,
            Child,
            Blackboard,
        }

        match [Choice::Id, Choice::Child, Choice::Blackboard]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::Id => BehaviorTree::Id(self.id().generate(rng)),
            Choice::Child => BehaviorTree::Child(generate_boxed_tree_node(self.child(), rng)),
            Choice::Blackboard => BehaviorTree::Blackboard(self.blackboard().generate(rng)),
        }
    }
}

impl OpGeneratorNested for TreeNodeKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            TreeNodeKindContainer::Unset => {
                match [0_u8, 1, 2, 3].into_iter().choose(rng).unwrap() {
                    0 => TreeNodeKind::ExecutionNode(ExecutionNodeKindLog::default().generate(rng)),
                    1 => TreeNodeKind::Decorator(DecoratorKindLog::default().generate(rng)),
                    2 => TreeNodeKind::ControlNode(ControlNodeKindLog::default().generate(rng)),
                    _ => TreeNodeKind::SubTree(SubTreeLog::default().generate(rng)),
                }
            }
            TreeNodeKindContainer::Value(child) => match child.as_ref() {
                TreeNodeKindChild::ExecutionNode(log) => {
                    TreeNodeKind::ExecutionNode(log.generate(rng))
                }
                TreeNodeKindChild::Decorator(log) => TreeNodeKind::Decorator(log.generate(rng)),
                TreeNodeKindChild::ControlNode(log) => TreeNodeKind::ControlNode(log.generate(rng)),
                TreeNodeKindChild::SubTree(log) => TreeNodeKind::SubTree(log.generate(rng)),
            },
            TreeNodeKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    TreeNodeKindChild::ExecutionNode(log) => {
                        TreeNodeKind::ExecutionNode(log.generate(rng))
                    }
                    TreeNodeKindChild::Decorator(log) => TreeNodeKind::Decorator(log.generate(rng)),
                    TreeNodeKindChild::ControlNode(log) => {
                        TreeNodeKind::ControlNode(log.generate(rng))
                    }
                    TreeNodeKindChild::SubTree(log) => TreeNodeKind::SubTree(log.generate(rng)),
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for TreeNodeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Id,
            Name,
        }

        match [Choice::Id, Choice::Name].into_iter().choose(rng).unwrap() {
            Choice::Id => TreeNode::Id(self.id().generate(rng)),
            Choice::Name => TreeNode::Name(self.name().generate(rng)),
        }
    }
}

impl OpGeneratorNested for BlackboardLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Blackboard::Entries(self.entries().generate(rng))
    }
}

impl OpGeneratorNested for ExecutionNodeKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ExecutionNodeKindContainer::Unset => match [0_u8, 1].into_iter().choose(rng).unwrap() {
                0 => ExecutionNodeKind::Action(ActionKindLog::default().generate(rng)),
                _ => ExecutionNodeKind::Condition(ConditionKindLog::default().generate(rng)),
            },
            ExecutionNodeKindContainer::Value(child) => match child.as_ref() {
                ExecutionNodeKindChild::Action(log) => ExecutionNodeKind::Action(log.generate(rng)),
                ExecutionNodeKindChild::Condition(log) => {
                    ExecutionNodeKind::Condition(log.generate(rng))
                }
            },
            ExecutionNodeKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ExecutionNodeKindChild::Action(log) => {
                        ExecutionNodeKind::Action(log.generate(rng))
                    }
                    ExecutionNodeKindChild::Condition(log) => {
                        ExecutionNodeKind::Condition(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ExecutionNodeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNode,
            Outflowports,
            Inflowports,
        }

        match [Choice::TreeNode, Choice::Outflowports, Choice::Inflowports]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNode => ExecutionNode::TreeNodeSuper(self.tree_node_super().generate(rng)),
            Choice::Outflowports => ExecutionNode::Outflowports(self.outflowports().generate(rng)),
            Choice::Inflowports => ExecutionNode::Inflowports(self.inflowports().generate(rng)),
        }
    }
}

impl OpGeneratorNested for DataFlowPortKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            DataFlowPortKindContainer::Unset => match [0_u8, 1].into_iter().choose(rng).unwrap() {
                0 => DataFlowPortKind::OutFlowPort(OutFlowPortLog::default().generate(rng)),
                _ => DataFlowPortKind::InFlowPort(InFlowPortLog::default().generate(rng)),
            },
            DataFlowPortKindContainer::Value(child) => match child.as_ref() {
                DataFlowPortKindChild::OutFlowPort(log) => {
                    DataFlowPortKind::OutFlowPort(log.generate(rng))
                }
                DataFlowPortKindChild::InFlowPort(log) => {
                    DataFlowPortKind::InFlowPort(log.generate(rng))
                }
            },
            DataFlowPortKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    DataFlowPortKindChild::OutFlowPort(log) => {
                        DataFlowPortKind::OutFlowPort(log.generate(rng))
                    }
                    DataFlowPortKindChild::InFlowPort(log) => {
                        DataFlowPortKind::InFlowPort(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for DataFlowPortLog {
    fn generate(&self, _rng: &mut impl Rng) -> Self::Op {
        DataFlowPort::New
    }
}

impl OpGeneratorNested for OutFlowPortLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        OutFlowPort::DataFlowPortSuper(self.data_flow_port_super().generate(rng))
    }
}

impl OpGeneratorNested for InFlowPortLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        InFlowPort::DataFlowPortSuper(self.data_flow_port_super().generate(rng))
    }
}

impl OpGeneratorNested for DecoratorKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        let log = match &self.child {
            DecoratorKindContainer::Value(child) => match child.as_ref() {
                DecoratorKindChild::Inverter(log) => log.clone(),
            },
            DecoratorKindContainer::Conflicts(children) => children
                .iter()
                .map(|child| match child {
                    DecoratorKindChild::Inverter(log) => log.clone(),
                })
                .next()
                .unwrap_or_default(),
            DecoratorKindContainer::Unset => InverterLog::default(),
        };
        DecoratorKind::Inverter(log.generate(rng))
    }
}

impl OpGeneratorNested for DecoratorLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNode,
            Child,
        }

        match [Choice::TreeNode, Choice::Child]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNode => Decorator::TreeNodeSuper(self.tree_node_super().generate(rng)),
            Choice::Child => Decorator::Child(generate_boxed_tree_node(self.child(), rng)),
        }
    }
}

impl OpGeneratorNested for ControlNodeKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ControlNodeKindContainer::Unset => match [0_u8, 1].into_iter().choose(rng).unwrap() {
                0 => ControlNodeKind::Sequence(SequenceLog::default().generate(rng)),
                _ => ControlNodeKind::Fallback(FallbackLog::default().generate(rng)),
            },
            ControlNodeKindContainer::Value(child) => match child.as_ref() {
                ControlNodeKindChild::Sequence(log) => ControlNodeKind::Sequence(log.generate(rng)),
                ControlNodeKindChild::Fallback(log) => ControlNodeKind::Fallback(log.generate(rng)),
            },
            ControlNodeKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ControlNodeKindChild::Sequence(log) => {
                        ControlNodeKind::Sequence(log.generate(rng))
                    }
                    ControlNodeKindChild::Fallback(log) => {
                        ControlNodeKind::Fallback(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ControlNodeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNode,
            Children,
        }

        match [Choice::TreeNode, Choice::Children]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNode => ControlNode::TreeNodeSuper(self.tree_node_super().generate(rng)),
            Choice::Children => {
                ControlNode::Children(generate_boxed_tree_list(self.children(), rng))
            }
        }
    }
}

impl OpGeneratorNested for SequenceLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Sequence::ControlNodeSuper(self.control_node_super().generate(rng))
    }
}

impl OpGeneratorNested for FallbackLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Fallback::ControlNodeSuper(self.control_node_super().generate(rng))
    }
}

impl OpGeneratorNested for ActionKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ActionKindContainer::Unset => match [0_u8, 1, 2].into_iter().choose(rng).unwrap() {
                0 => ActionKind::OpenDoor(OpenDoorLog::default().generate(rng)),
                1 => ActionKind::EnterRoom(EnterRoomLog::default().generate(rng)),
                _ => ActionKind::CloseDoor(CloseDoorLog::default().generate(rng)),
            },
            ActionKindContainer::Value(child) => match child.as_ref() {
                ActionKindChild::OpenDoor(log) => ActionKind::OpenDoor(log.generate(rng)),
                ActionKindChild::EnterRoom(log) => ActionKind::EnterRoom(log.generate(rng)),
                ActionKindChild::CloseDoor(log) => ActionKind::CloseDoor(log.generate(rng)),
            },
            ActionKindContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ActionKindChild::OpenDoor(log) => ActionKind::OpenDoor(log.generate(rng)),
                    ActionKindChild::EnterRoom(log) => ActionKind::EnterRoom(log.generate(rng)),
                    ActionKindChild::CloseDoor(log) => ActionKind::CloseDoor(log.generate(rng)),
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ActionLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Action::ExecutionNodeSuper(self.execution_node_super().generate(rng))
    }
}

impl OpGeneratorNested for ConditionKindLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        let log = match &self.child {
            ConditionKindContainer::Value(child) => match child.as_ref() {
                ConditionKindChild::IsDoorOpen(log) => log.clone(),
            },
            ConditionKindContainer::Conflicts(children) => children
                .iter()
                .map(|child| match child {
                    ConditionKindChild::IsDoorOpen(log) => log.clone(),
                })
                .next()
                .unwrap_or_default(),
            ConditionKindContainer::Unset => IsDoorOpenLog::default(),
        };
        ConditionKind::IsDoorOpen(log.generate(rng))
    }
}

impl OpGeneratorNested for ConditionLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Condition::ExecutionNodeSuper(self.execution_node_super().generate(rng))
    }
}

impl OpGeneratorNested for BlackboardEntryLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Key,
            Value,
        }

        match [Choice::Key, Choice::Value]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::Key => BlackboardEntry::Key(self.key().generate(rng)),
            Choice::Value => BlackboardEntry::Value(self.value().generate(rng)),
        }
    }
}

impl OpGeneratorNested for InverterLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Inverter::DecoratorSuper(self.decorator_super().generate(rng))
    }
}

impl OpGeneratorNested for IsDoorOpenLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        IsDoorOpen::ConditionSuper(self.condition_super().generate(rng))
    }
}

impl OpGeneratorNested for OpenDoorLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        OpenDoor::ActionSuper(self.action_super().generate(rng))
    }
}

impl OpGeneratorNested for EnterRoomLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        EnterRoom::ActionSuper(self.action_super().generate(rng))
    }
}

impl OpGeneratorNested for CloseDoorLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        CloseDoor::ActionSuper(self.action_super().generate(rng))
    }
}

impl OpGeneratorNested for SubTreeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNode,
            Tree,
        }

        match [Choice::TreeNode, Choice::Tree]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNode => SubTree::TreeNodeSuper(self.tree_node_super().generate(rng)),
            Choice::Tree => SubTree::Tree(self.tree().generate(rng)),
        }
    }
}
impl FuzzMetrics for BehaviortreeLog {
    fn structure_metrics(&self) -> StructureMetrics {
        StructureMetrics::object([self.root_log().structure_metrics()])
    }
}
