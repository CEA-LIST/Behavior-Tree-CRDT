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

fn generate_boxed_tree_node(log: &Box<TreeNodeLog>, rng: &mut impl Rng) -> Box<TreeNode> {
    Box::new(log.as_ref().generate(rng))
}

fn generate_boxed_tree_list(
    log: &NestedListLog<Box<TreeNodeLog>>,
    rng: &mut impl Rng,
) -> NestedList<Box<TreeNode>> {
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
            let value = generate_boxed_tree_node(&Box::<TreeNodeLog>::default(), rng);
            NestedList::Insert { pos, value }
        }
        Choice::Update => {
            let pos = rng.random_range(0..positions.len());
            let target_id = &positions[pos];
            let value = log
                .children()
                .get(target_id)
                .map(|child| generate_boxed_tree_node(child, rng))
                .unwrap_or_else(|| generate_boxed_tree_node(&Box::<TreeNodeLog>::default(), rng));
            NestedList::Update { pos, value }
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

impl OpGeneratorNested for TreeNodeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            TreeNodeContainer::Unset => match [0_u8, 1, 2, 3].into_iter().choose(rng).unwrap() {
                0 => TreeNode::ExecutionNode(ExecutionNodeLog::default().generate(rng)),
                1 => TreeNode::Decorator(DecoratorLog::default().generate(rng)),
                2 => TreeNode::ControlNode(ControlNodeLog::default().generate(rng)),
                _ => TreeNode::SubTree(SubTreeLog::default().generate(rng)),
            },
            TreeNodeContainer::Value(child) => match child.as_ref() {
                TreeNodeChild::ExecutionNode(log) => TreeNode::ExecutionNode(log.generate(rng)),
                TreeNodeChild::Decorator(log) => TreeNode::Decorator(log.generate(rng)),
                TreeNodeChild::ControlNode(log) => TreeNode::ControlNode(log.generate(rng)),
                TreeNodeChild::SubTree(log) => TreeNode::SubTree(log.generate(rng)),
            },
            TreeNodeContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    TreeNodeChild::ExecutionNode(log) => TreeNode::ExecutionNode(log.generate(rng)),
                    TreeNodeChild::Decorator(log) => TreeNode::Decorator(log.generate(rng)),
                    TreeNodeChild::ControlNode(log) => TreeNode::ControlNode(log.generate(rng)),
                    TreeNodeChild::SubTree(log) => TreeNode::SubTree(log.generate(rng)),
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for TreeNodeFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            Id,
            Name,
        }

        match [Choice::Id, Choice::Name].into_iter().choose(rng).unwrap() {
            Choice::Id => TreeNodeFeat::Id(self.id().generate(rng)),
            Choice::Name => TreeNodeFeat::Name(self.name().generate(rng)),
        }
    }
}

impl OpGeneratorNested for BlackboardLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Blackboard::Entries(self.entries().generate(rng))
    }
}

impl OpGeneratorNested for ExecutionNodeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ExecutionNodeContainer::Unset => match [0_u8, 1].into_iter().choose(rng).unwrap() {
                0 => ExecutionNode::Action(ActionLog::default().generate(rng)),
                _ => ExecutionNode::Condition(ConditionLog::default().generate(rng)),
            },
            ExecutionNodeContainer::Value(child) => match child.as_ref() {
                ExecutionNodeChild::Action(log) => ExecutionNode::Action(log.generate(rng)),
                ExecutionNodeChild::Condition(log) => ExecutionNode::Condition(log.generate(rng)),
            },
            ExecutionNodeContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ExecutionNodeChild::Action(log) => ExecutionNode::Action(log.generate(rng)),
                    ExecutionNodeChild::Condition(log) => {
                        ExecutionNode::Condition(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ExecutionNodeFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNodeFeat,
            Outflowports,
            Inflowports,
        }

        match [
            Choice::TreeNodeFeat,
            Choice::Outflowports,
            Choice::Inflowports,
        ]
        .into_iter()
        .choose(rng)
        .unwrap()
        {
            Choice::TreeNodeFeat => {
                ExecutionNodeFeat::TreeNodeFeat(self.tree_node_feat().generate(rng))
            }
            Choice::Outflowports => {
                ExecutionNodeFeat::Outflowports(self.outflowports().generate(rng))
            }
            Choice::Inflowports => ExecutionNodeFeat::Inflowports(self.inflowports().generate(rng)),
        }
    }
}

impl OpGeneratorNested for DataFlowPortLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            DataFlowPortContainer::Unset => match [0_u8, 1].into_iter().choose(rng).unwrap() {
                0 => DataFlowPort::OutFlowPort(OutFlowPortLog::default().generate(rng)),
                _ => DataFlowPort::InFlowPort(InFlowPortLog::default().generate(rng)),
            },
            DataFlowPortContainer::Value(child) => match child.as_ref() {
                DataFlowPortChild::OutFlowPort(log) => DataFlowPort::OutFlowPort(log.generate(rng)),
                DataFlowPortChild::InFlowPort(log) => DataFlowPort::InFlowPort(log.generate(rng)),
            },
            DataFlowPortContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    DataFlowPortChild::OutFlowPort(log) => {
                        DataFlowPort::OutFlowPort(log.generate(rng))
                    }
                    DataFlowPortChild::InFlowPort(log) => {
                        DataFlowPort::InFlowPort(log.generate(rng))
                    }
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for DataFlowPortFeatLog {
    fn generate(&self, _rng: &mut impl Rng) -> Self::Op {
        DataFlowPortFeat::New
    }
}

impl OpGeneratorNested for OutFlowPortLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        OutFlowPort::DataFlowPortFeat(self.data_flow_port_feat().generate(rng))
    }
}

impl OpGeneratorNested for InFlowPortLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        InFlowPort::DataFlowPortFeat(self.data_flow_port_feat().generate(rng))
    }
}

impl OpGeneratorNested for DecoratorLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        let log = match &self.child {
            DecoratorContainer::Value(child) => match child.as_ref() {
                DecoratorChild::Inverter(log) => log.clone(),
            },
            DecoratorContainer::Conflicts(children) => children
                .iter()
                .find_map(|child| match child {
                    DecoratorChild::Inverter(log) => Some(log.clone()),
                })
                .unwrap_or_default(),
            DecoratorContainer::Unset => InverterLog::default(),
        };
        Decorator::Inverter(log.generate(rng))
    }
}

impl OpGeneratorNested for DecoratorFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNodeFeat,
            Child,
        }

        match [Choice::TreeNodeFeat, Choice::Child]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNodeFeat => {
                DecoratorFeat::TreeNodeFeat(self.tree_node_feat().generate(rng))
            }
            Choice::Child => DecoratorFeat::Child(generate_boxed_tree_node(self.child(), rng)),
        }
    }
}

impl OpGeneratorNested for ControlNodeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ControlNodeContainer::Unset => match [0_u8, 1].into_iter().choose(rng).unwrap() {
                0 => ControlNode::Sequence(SequenceLog::default().generate(rng)),
                _ => ControlNode::Fallback(FallbackLog::default().generate(rng)),
            },
            ControlNodeContainer::Value(child) => match child.as_ref() {
                ControlNodeChild::Sequence(log) => ControlNode::Sequence(log.generate(rng)),
                ControlNodeChild::Fallback(log) => ControlNode::Fallback(log.generate(rng)),
            },
            ControlNodeContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ControlNodeChild::Sequence(log) => ControlNode::Sequence(log.generate(rng)),
                    ControlNodeChild::Fallback(log) => ControlNode::Fallback(log.generate(rng)),
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ControlNodeFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNodeFeat,
            Children,
        }

        match [Choice::TreeNodeFeat, Choice::Children]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNodeFeat => {
                ControlNodeFeat::TreeNodeFeat(self.tree_node_feat().generate(rng))
            }
            Choice::Children => {
                ControlNodeFeat::Children(generate_boxed_tree_list(self.children(), rng))
            }
        }
    }
}

impl OpGeneratorNested for SequenceLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Sequence::ControlNodeFeat(self.control_node_feat().generate(rng))
    }
}

impl OpGeneratorNested for FallbackLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        Fallback::ControlNodeFeat(self.control_node_feat().generate(rng))
    }
}

impl OpGeneratorNested for ActionLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        match &self.child {
            ActionContainer::Unset => match [0_u8, 1, 2].into_iter().choose(rng).unwrap() {
                0 => Action::OpenDoor(OpenDoorLog::default().generate(rng)),
                1 => Action::EnterRoom(EnterRoomLog::default().generate(rng)),
                _ => Action::CloseDoor(CloseDoorLog::default().generate(rng)),
            },
            ActionContainer::Value(child) => match child.as_ref() {
                ActionChild::OpenDoor(log) => Action::OpenDoor(log.generate(rng)),
                ActionChild::EnterRoom(log) => Action::EnterRoom(log.generate(rng)),
                ActionChild::CloseDoor(log) => Action::CloseDoor(log.generate(rng)),
            },
            ActionContainer::Conflicts(children) => children
                .iter()
                .choose(rng)
                .map(|child| match child {
                    ActionChild::OpenDoor(log) => Action::OpenDoor(log.generate(rng)),
                    ActionChild::EnterRoom(log) => Action::EnterRoom(log.generate(rng)),
                    ActionChild::CloseDoor(log) => Action::CloseDoor(log.generate(rng)),
                })
                .unwrap(),
        }
    }
}

impl OpGeneratorNested for ActionFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        ActionFeat::ExecutionNodeFeat(self.execution_node_feat().generate(rng))
    }
}

impl OpGeneratorNested for ConditionLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        let log = match &self.child {
            ConditionContainer::Value(child) => match child.as_ref() {
                ConditionChild::IsDoorOpen(log) => log.clone(),
            },
            ConditionContainer::Conflicts(children) => children
                .iter()
                .find_map(|child| match child {
                    ConditionChild::IsDoorOpen(log) => Some(log.clone()),
                })
                .unwrap_or_default(),
            ConditionContainer::Unset => IsDoorOpenLog::default(),
        };
        Condition::IsDoorOpen(log.generate(rng))
    }
}

impl OpGeneratorNested for ConditionFeatLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        ConditionFeat::ExecutionNodeFeat(self.execution_node_feat().generate(rng))
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
        Inverter::DecoratorFeat(self.decorator_feat().generate(rng))
    }
}

impl OpGeneratorNested for IsDoorOpenLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        IsDoorOpen::ConditionFeat(self.condition_feat().generate(rng))
    }
}

impl OpGeneratorNested for OpenDoorLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        OpenDoor::ActionFeat(self.action_feat().generate(rng))
    }
}

impl OpGeneratorNested for EnterRoomLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        EnterRoom::ActionFeat(self.action_feat().generate(rng))
    }
}

impl OpGeneratorNested for CloseDoorLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        CloseDoor::ActionFeat(self.action_feat().generate(rng))
    }
}

impl OpGeneratorNested for SubTreeLog {
    fn generate(&self, rng: &mut impl Rng) -> Self::Op {
        #[derive(Clone, Copy)]
        enum Choice {
            TreeNodeFeat,
            Tree,
        }

        match [Choice::TreeNodeFeat, Choice::Tree]
            .into_iter()
            .choose(rng)
            .unwrap()
        {
            Choice::TreeNodeFeat => SubTree::TreeNodeFeat(self.tree_node_feat().generate(rng)),
            Choice::Tree => SubTree::Tree(self.tree().generate(rng)),
        }
    }
}
impl FuzzMetrics for BehaviortreeLog {
    fn structure_metrics(&self) -> StructureMetrics {
        StructureMetrics::object([self.root_log().structure_metrics()])
    }
}
