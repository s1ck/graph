use std::path::PathBuf;

use graph::prelude::*;

#[test]
fn should_compile_test() {
    fn inner_test() -> Result<(), Error> {
        let _g: DirectedCsrGraph<usize> = GraphBuilder::new()
            .file_format(EdgeListInput::default())
            .path("graph")
            .build()?;

        let _g: DirectedCsrGraph<_> = GraphBuilder::new()
            .file_format(EdgeListInput::<usize>::default())
            .path("graph")
            .build()?;

        let _g: UndirectedCsrGraph<usize> = GraphBuilder::new()
            .file_format(EdgeListInput::default())
            .path("graph")
            .build()?;

        let _g: DirectedCsrGraph<usize> = GraphBuilder::new()
            .file_format(BinaryInput::<usize>::default())
            .path("graph")
            .build()?;

        let _g: DirectedCsrGraph<usize> = GraphBuilder::new()
            .file_format(DotGraphInput::<usize, usize>::default())
            .path("graph")
            .build()?;

        let _g: UndirectedCsrGraph<usize> = GraphBuilder::new()
            .file_format(DotGraphInput::<usize, usize>::default())
            .path("graph")
            .build()?;

        let _g: DirectedNodeLabeledCsrGraph<usize, usize> = GraphBuilder::new()
            .file_format(DotGraphInput::default())
            .path("graph")
            .build()?;

        let _g: UndirectedNodeLabeledCsrGraph<usize, usize> = GraphBuilder::new()
            .file_format(DotGraphInput::default())
            .path("graph")
            .build()?;

        Ok(())
    }

    assert!(inner_test().is_err())
}

#[test]
fn directed_usize_graph_from_edge_list() {
    assert_directed_graph::<usize>(
        GraphBuilder::new()
            .edges([(0, 1), (0, 2), (1, 2), (1, 3), (2, 4), (3, 4)])
            .build(),
    );
}

#[test]
fn directed_u32_graph_from_edge_list() {
    assert_directed_graph::<u32>(
        GraphBuilder::new()
            .edges([(0, 1), (0, 2), (1, 2), (1, 3), (2, 4), (3, 4)])
            .build(),
    );
}

#[test]
fn directed_usize_graph_from_edge_list_with_values() {
    let graph: DirectedCsrGraph<usize, f32> = GraphBuilder::new()
        .edges_with_values([
            (0, 1, 0.1),
            (0, 2, 0.2),
            (1, 2, 0.3),
            (1, 3, 0.4),
            (2, 4, 0.5),
            (3, 4, 0.6),
        ])
        .build();

    assert_eq!(
        graph.out_neighbors_with_values(0),
        &[Target::new(1, 0.1), Target::new(2, 0.2)]
    );

    assert_eq!(
        graph.in_neighbors_with_values(2),
        &[Target::new(0, 0.2), Target::new(1, 0.3)]
    );
}

#[test]
fn directed_usize_graph_from_gdl() {
    assert_directed_graph::<usize>(
        GraphBuilder::new()
            .gdl_str::<usize, _>(
                "(n0)-->(n1),(n0)-->(n2),(n1)-->(n2),(n1)-->(n3),(n2)-->(n4),(n3)-->(n4)",
            )
            .build()
            .unwrap(),
    )
}

#[test]
fn directed_usize_graph_from_gdl_with_f32_edge_values() {
    let g: DirectedCsrGraph<usize, f32> = GraphBuilder::new()
        .gdl_str::<usize, _>(
            "(n0)-[{ f: 0.1 }]->(n1),
                 (n0)-[{ f: 0.2 }]->(n2),
                 (n1)-[{ f: 0.3 }]->(n2),
                 (n1)-[{ f: 0.4 }]->(n3),
                 (n2)-[{ f: 0.5 }]->(n4),
                 (n3)-[{ f: 0.6 }]->(n4)",
        )
        .build()
        .unwrap();

    let actual = g.out_neighbors_with_values(0);

    assert_eq!(actual, &[Target::new(1, 0.1), Target::new(2, 0.2)]);
}

#[test]
fn directed_usize_graph_from_gdl_with_i64_edge_values() {
    let g: DirectedCsrGraph<usize, i64> = GraphBuilder::new()
        .gdl_str::<usize, _>(
            "(n0)-[{ f: 42 }]->(n1),
                 (n0)-[{ f: 43 }]->(n2),
                 (n1)-[{ f: 44 }]->(n2),
                 (n1)-[{ f: 45 }]->(n3),
                 (n2)-[{ f: 46 }]->(n4),
                 (n3)-[{ f: 47 }]->(n4)",
        )
        .build()
        .unwrap();

    let actual = g.out_neighbors_with_values(0);

    assert_eq!(actual, &[Target::new(1, 42), Target::new(2, 43)]);
}

#[test]
fn undirected_usize_graph_from_gdl() {
    assert_undirected_graph::<usize>(
        GraphBuilder::new()
            .gdl_str::<usize, _>(
                "(n0)-->(n1),(n0)-->(n2),(n1)-->(n2),(n1)-->(n3),(n2)-->(n4),(n3)-->(n4)",
            )
            .build()
            .unwrap(),
    )
}

#[test]
fn undirected_usize_graph_from_edge_list() {
    assert_undirected_graph::<usize>(
        GraphBuilder::new()
            .edges([(0, 1), (0, 2), (1, 2), (1, 3), (2, 4), (3, 4)])
            .build(),
    );
}

#[test]
fn undirected_u32_graph_from_edge_list() {
    assert_undirected_graph::<u32>(
        GraphBuilder::new()
            .edges([(0, 1), (0, 2), (1, 2), (1, 3), (2, 4), (3, 4)])
            .build(),
    );
}

#[test]
fn undirected_usize_graph_from_edge_list_with_values() {
    let graph: UndirectedCsrGraph<usize, f32> = GraphBuilder::new()
        .edges_with_values([
            (0, 1, 0.1),
            (0, 2, 0.2),
            (1, 2, 0.3),
            (1, 3, 0.4),
            (2, 4, 0.5),
            (3, 4, 0.6),
        ])
        .build();

    assert_eq!(
        graph.neighbors_with_values(1),
        &[
            Target::new(0, 0.1),
            Target::new(2, 0.3),
            Target::new(3, 0.4)
        ]
    );
}

#[test]
fn directed_usize_graph_from_edge_list_file() {
    let path = [env!("CARGO_MANIFEST_DIR"), "resources", "test.el"]
        .iter()
        .collect::<PathBuf>();

    let graph = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .file_format(EdgeListInput::default())
        .path(path)
        .build()
        .expect("loading failed");

    assert_directed_graph::<usize>(graph);
}

#[test]
fn directed_u32_graph_from_edge_list_file() {
    let path = [env!("CARGO_MANIFEST_DIR"), "resources", "test.el"]
        .iter()
        .collect::<PathBuf>();

    let graph = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .file_format(EdgeListInput::default())
        .path(path)
        .build()
        .expect("loading failed");

    assert_directed_graph::<u32>(graph);
}

#[test]
fn directed_u32_graph_from_dot_graph_file() {
    let path = [env!("CARGO_MANIFEST_DIR"), "resources", "test.graph"]
        .iter()
        .collect::<PathBuf>();

    let graph = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .file_format(DotGraphInput::<u32, u32>::default())
        .path(path)
        .build()
        .expect("loading failed");

    assert_directed_graph::<u32>(graph);
}

#[test]
fn undirected_usize_graph_from_edge_list_file() {
    let path = [env!("CARGO_MANIFEST_DIR"), "resources", "test.el"]
        .iter()
        .collect::<PathBuf>();

    let graph = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .file_format(EdgeListInput::default())
        .path(path)
        .build()
        .expect("loading failed");

    assert_undirected_graph::<usize>(graph);
}

#[test]
fn undirected_u32_graph_from_edge_list_file() {
    let path = [env!("CARGO_MANIFEST_DIR"), "resources", "test.el"]
        .iter()
        .collect::<PathBuf>();

    let graph = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .file_format(EdgeListInput::default())
        .path(path)
        .build()
        .expect("loading failed");

    assert_undirected_graph::<u32>(graph);
}

#[test]
fn undirected_u32_graph_from_dot_graph_file() {
    let path = [env!("CARGO_MANIFEST_DIR"), "resources", "test.graph"]
        .iter()
        .collect::<PathBuf>();

    let graph = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .file_format(DotGraphInput::<u32, u32>::default())
        .path(path)
        .build()
        .expect("loading failed");

    assert_undirected_graph::<u32>(graph);
}

fn assert_directed_graph<NI: Idx>(g: DirectedCsrGraph<NI>) {
    assert_eq!(g.node_count(), NI::new(5));
    assert_eq!(g.edge_count(), NI::new(6));

    assert_eq!(g.out_degree(NI::new(0)), NI::new(2));
    assert_eq!(g.out_degree(NI::new(1)), NI::new(2));
    assert_eq!(g.out_degree(NI::new(2)), NI::new(1));
    assert_eq!(g.out_degree(NI::new(3)), NI::new(1));
    assert_eq!(g.out_degree(NI::new(4)), NI::new(0));

    assert_eq!(g.in_degree(NI::new(0)), NI::new(0));
    assert_eq!(g.in_degree(NI::new(1)), NI::new(1));
    assert_eq!(g.in_degree(NI::new(2)), NI::new(2));
    assert_eq!(g.in_degree(NI::new(3)), NI::new(1));
    assert_eq!(g.in_degree(NI::new(4)), NI::new(2));

    assert_eq!(g.out_neighbors(NI::new(0)), &[NI::new(1), NI::new(2)]);
    assert_eq!(g.out_neighbors(NI::new(1)), &[NI::new(2), NI::new(3)]);
    assert_eq!(g.out_neighbors(NI::new(2)), &[NI::new(4)]);
    assert_eq!(g.out_neighbors(NI::new(3)), &[NI::new(4)]);
    assert_eq!(g.out_neighbors(NI::new(4)), &[]);

    assert_eq!(g.in_neighbors(NI::new(0)), &[]);
    assert_eq!(g.in_neighbors(NI::new(1)), &[NI::new(0)]);
    assert_eq!(g.in_neighbors(NI::new(2)), &[NI::new(0), NI::new(1)]);
    assert_eq!(g.in_neighbors(NI::new(3)), &[NI::new(1)]);
    assert_eq!(g.in_neighbors(NI::new(4)), &[NI::new(2), NI::new(3)]);
}

fn assert_undirected_graph<NI: Idx>(g: UndirectedCsrGraph<NI>) {
    assert_eq!(g.node_count(), NI::new(5));
    assert_eq!(g.edge_count(), NI::new(6));

    assert_eq!(g.degree(NI::new(0)), NI::new(2));
    assert_eq!(g.degree(NI::new(1)), NI::new(3));
    assert_eq!(g.degree(NI::new(2)), NI::new(3));
    assert_eq!(g.degree(NI::new(3)), NI::new(2));
    assert_eq!(g.degree(NI::new(4)), NI::new(2));

    assert_eq!(g.neighbors(NI::new(0)), &[NI::new(1), NI::new(2)]);
    assert_eq!(
        g.neighbors(NI::new(1)),
        &[NI::new(0), NI::new(2), NI::new(3)]
    );
    assert_eq!(
        g.neighbors(NI::new(2)),
        &[NI::new(0), NI::new(1), NI::new(4)]
    );
    assert_eq!(g.neighbors(NI::new(3)), &[NI::new(1), NI::new(4)]);
    assert_eq!(g.neighbors(NI::new(4)), &[NI::new(2), NI::new(3)]);
}
