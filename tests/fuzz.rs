use behaviortree::{package::BehaviortreeLog, utils::graph_view::Vf2GraphView};

#[test]
fn fuzz() {
    use moirai_fuzz::{
        config::{FuzzerConfig, RunConfig},
        fuzzer::fuzzer,
    };

    let run = RunConfig::new(0.4, 4, 1_000, None, None, true, false);
    let runs = vec![run.clone(); 1];

    let config = FuzzerConfig::<BehaviortreeLog>::new(
        "bt",
        runs,
        true,
        |a, b| {
            let package = a.root == b.root;
            if !package {
                println!("Package mismatch");
                println!("----- Root A -----");
                println!("{:#?}", a.root);
                println!("----- Root B -----");
                println!("{:#?}", b.root);
                return false;
            }

            if a.refs.node_count() == 0 && b.refs.node_count() == 0 {
                // If both graphs are empty, skip the isomorphism check to avoid false negatives due to different node IDs.
                return true;
            } else {
                let refs = vf2::isomorphisms(&Vf2GraphView(&a.refs), &Vf2GraphView(&b.refs))
                    .default_eq()
                    .first()
                    .is_some();
                if !refs {
                    println!(
                        "Graph isomorphism mismatch: nodes {} vs {}, edges {} vs {}",
                        a.refs.node_count(),
                        b.refs.node_count(),
                        a.refs.edge_count(),
                        b.refs.edge_count()
                    );
                    println!("----- Graph A -----");
                    println!("{:#?}", a.refs);
                    println!("----- Graph B -----");
                    println!("{:#?}", b.refs);
                }
                return refs;
            }
        },
        false,
    );

    fuzzer::<BehaviortreeLog>(config);
}
