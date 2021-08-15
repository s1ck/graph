use log::info;
use rayon::prelude::*;

use crate::graph::csr::{prefix_sum, Csr};
use crate::index::Idx;
use crate::{DirectedGraph, Error, Graph, SharedMut, UndirectedGraph};

use std::ops::Range;
use std::sync::Arc;
use std::time::Instant;

/// Partition the node set based on the degrees of the nodes.
pub trait DegreePartitionOp<Node: Idx> {
    /// Creates a range-based degree partition of the nodes.
    ///
    /// Divide the nodes into `concurrency` number of ranges such that these
    /// ranges have roughly equal total degree. That is, the sum of the degrees
    /// of the nodes of each range should be roughly equal to the extent that
    /// that's actually possible.
    /// The length of the returned vector will never exceed `concurrency`.
    fn degree_partition(&self, concurrency: usize) -> Vec<Range<Node>>;
}

/// Partition the node set based on the out degrees of the nodes.
pub trait OutDegreePartitionOp<Node: Idx> {
    /// Creates a range-based out degree partition of the nodes.
    ///
    /// Divide the nodes into `concurrency` number of ranges such that these
    /// ranges have roughly equal total out degree. That is, the sum of the out
    /// degrees of the nodes of each range should be roughly equal to the extent
    /// that that's actually possible.
    /// The length of the returned vector will never exceed `concurrency`.
    fn out_degree_partition(&self, concurrency: usize) -> Vec<Range<Node>>;
}

/// Partition the node set based on the in degrees of the nodes.
pub trait InDegreePartitionOp<Node: Idx> {
    /// Creates a range-based in degree partition of the nodes.
    ///
    /// Divide the nodes into `concurrency` number of ranges such that these
    /// ranges have roughly equal total in degree. That is, the sum of the in
    /// degrees of the nodes of each range should be roughly equal to the extent
    /// that that's actually possible.
    /// The length of the returned vector will never exceed `concurrency`.
    fn in_degree_partition(&self, concurrency: usize) -> Vec<Range<Node>>;
}

/// Call a particular function for each node with its corresponding state.
pub trait ForEachNodeOp<Node: Idx> {
    /// For each node calls `node_fn` with the node and its corresponding
    /// mutable state.
    ///
    /// For every node `n` in the graph `node_fn(&self, n, node_values[n.index()])`
    /// will be called.
    ///
    /// `node_values` must have length exactly equal to the number of nodes in
    /// the graph.
    ///
    /// # Example
    ///
    /// ```
    /// # use graph::prelude::*;
    /// # use std::ops::Range;
    /// let graph: DirectedCsrGraph<u32> = GraphBuilder::new()
    ///     .edges(vec![(0, 1), (0, 2), (1, 2)])
    ///     .build();
    /// let mut node_values = vec![0; 3];
    ///
    /// graph.
    ///     for_each_node(&mut node_values, |g, node, node_state| {
    ///         *node_state = g.out_degree(node);
    ///     });
    ///
    /// assert_eq!(node_values[0], 2);
    /// assert_eq!(node_values[1], 1);
    /// assert_eq!(node_values[2], 0);
    /// ```
    fn for_each_node<T, F>(&self, node_values: &mut [T], node_fn: F) -> Result<(), Error>
    where
        T: Send,
        F: Fn(&Self, Node, &mut T) + Send + Sync;
}

/// Call a particular function for each node with its corresponding state with partition hint.
pub trait ForEachNodeByPartitionOp<Node: Idx> {
    /// For each node calls `node_fn` with the node and its corresponding
    /// mutable state, using `partition` as a parallelization hint.
    ///
    /// For every node `n` in the graph `node_fn(&self, n, node_values[n.index()])`
    /// will be called.
    ///
    /// `node_values` must have length exactly equal to the number of nodes in
    /// the graph.
    ///
    /// A multithreaded implementation will base its parallelization scheme on
    /// the provided `partition`.
    ///
    /// # Example
    ///
    /// ```
    /// # use graph::prelude::*;
    /// # use std::ops::Range;
    /// let graph: DirectedCsrGraph<u32> = GraphBuilder::new()
    ///     .edges(vec![(0, 1), (0, 2), (1, 2)])
    ///     .build();
    /// let mut node_values = vec![0; 3];
    /// let partition: Vec<Range<u32>> = graph.out_degree_partition(num_cpus::get());
    ///
    /// graph.
    ///     for_each_node_by_partition(&partition, &mut node_values, |g, node, node_state| {
    ///         *node_state = g.out_degree(node);
    ///     });
    ///
    /// assert_eq!(node_values[0], 2);
    /// assert_eq!(node_values[1], 1);
    /// assert_eq!(node_values[2], 0);
    /// ```
    fn for_each_node_by_partition<T, F>(
        &self,
        partition: &[Range<Node>],
        node_values: &mut [T],
        node_fn: F,
    ) -> Result<(), Error>
    where
        T: Send,
        F: Fn(&Self, Node, &mut T) + Send + Sync;
}

pub trait RelabelByDegreeOp<Node: Idx> {
    /// Creates a new graph by relabeling the node ids of the given graph.
    ///
    /// Ids are relabaled using descending degree-order, i.e., given `n` nodes,
    /// the node with the largest degree will become node id `0`, the node with
    /// the smallest degree will become node id `n - 1`.
    ///
    /// Note, that this method creates a new graph with the same space
    /// requirements as the input graph.
    ///
    /// # Example
    ///
    /// ```
    /// use graph::prelude::*;
    ///
    /// let graph: UndirectedCsrGraph<u32> = GraphBuilder::new()
    ///     .edges(vec![(0, 1), (1, 2), (1, 3), (3, 0)])
    ///     .build();
    ///
    /// assert_eq!(graph.degree(0), 2);
    /// assert_eq!(graph.degree(1), 3);
    /// assert_eq!(graph.degree(2), 1);
    /// assert_eq!(graph.degree(3), 2);
    ///
    /// assert_eq!(graph.neighbors(0), &[1, 3]);
    ///
    /// let graph = graph.to_degree_ordered();
    ///
    /// assert_eq!(graph.degree(0), 3);
    /// assert_eq!(graph.degree(1), 2);
    /// assert_eq!(graph.degree(2), 2);
    /// assert_eq!(graph.degree(3), 1);
    ///
    /// assert_eq!(graph.neighbors(0), &[1, 2, 3]);
    /// ```
    fn to_degree_ordered(&self) -> Self;
}

pub trait SerializeGraphOp<W> {
    fn serialize(&self, write: W) -> Result<(), Error>;
}

pub trait DeserializeGraphOp<R, G> {
    fn deserialize(read: R) -> Result<G, Error>;
}

impl<Node, G> RelabelByDegreeOp<Node> for G
where
    Node: Idx,
    G: From<Csr<Node>> + UndirectedGraph<Node> + Sync,
{
    fn to_degree_ordered(&self) -> Self {
        relabel_by_degree(self)
    }
}

impl<Node, G> ForEachNodeOp<Node> for G
where
    Node: Idx,
    G: Graph<Node> + Sync,
{
    /// For each node calls a given function with the node and its corresponding
    /// mutable state in parallel.
    ///
    /// The parallelization is done by means of a [rayon](https://docs.rs/rayon/)
    /// based fork join with a task for each node.
    fn for_each_node<T, F>(&self, node_values: &mut [T], node_fn: F) -> Result<(), Error>
    where
        T: Send,
        F: Fn(&Self, Node, &mut T) + Send + Sync,
    {
        if node_values.len() != self.node_count().index() {
            return Err(Error::InvalidNodeValues);
        }

        let node_fn = Arc::new(node_fn);

        node_values
            .into_par_iter()
            .enumerate()
            .for_each(|(i, node_state)| node_fn(self, Node::new(i), node_state));

        Ok(())
    }
}

impl<Node, G> ForEachNodeByPartitionOp<Node> for G
where
    Node: Idx,
    G: Graph<Node> + Sync,
{
    /// For each node calls a given function with the node and its corresponding
    /// mutable state in parallel based on the provided node partition.
    ///
    /// The parallelization is done by means of a [rayon](https://docs.rs/rayon/)
    /// based fork join with a task for each range in the provided node partition.
    fn for_each_node_by_partition<T, F>(
        &self,
        partition: &[Range<Node>],
        node_values: &mut [T],
        node_fn: F,
    ) -> Result<(), Error>
    where
        T: Send,
        F: Fn(&Self, Node, &mut T) + Send + Sync,
    {
        if node_values.len() != self.node_count().index() {
            return Err(Error::InvalidNodeValues);
        }

        if partition.iter().map(|r| r.end - r.start).sum::<Node>() != self.node_count() {
            return Err(Error::InvalidPartitioning);
        }

        let node_fn = Arc::new(node_fn);

        let node_value_splits = split_by_partition(partition, node_values);

        node_value_splits
            .into_par_iter()
            .zip(partition.into_par_iter())
            .for_each_with(node_fn, |node_fn, (mutable_chunk, range)| {
                for (node_state, node) in mutable_chunk.iter_mut().zip(range.start..range.end) {
                    node_fn(self, node, node_state);
                }
            });

        Ok(())
    }
}

impl<Node: Idx, U: UndirectedGraph<Node>> DegreePartitionOp<Node> for U {
    /// Creates a greedy range-based degree partition of the nodes.
    ///
    /// It is greedy in the sense that it goes through the node set only once
    /// and simply adds a new range to the result whenever the current range's
    /// nodes' degrees sum up to at least the average node degree.
    ///
    /// # Example
    ///
    /// ```
    /// # use graph::prelude::*;
    /// # use std::ops::Range;
    /// let graph: UndirectedCsrGraph<u32> = GraphBuilder::new()
    ///     .edges(vec![(0, 1), (0, 2), (0, 3), (0, 3)])
    ///     .build();
    ///
    /// let partition: Vec<Range<u32>> = graph.degree_partition(2);
    ///
    /// assert_eq!(partition.len(), 2);
    /// assert_eq!(partition[0], 0..1);
    /// assert_eq!(partition[1], 1..4);
    /// ```
    fn degree_partition(&self, concurrency: usize) -> Vec<Range<Node>> {
        let batch_size = ((self.edge_count().index() * 2) as f64 / concurrency as f64).ceil();
        greedy_node_map_partition(
            |node| self.degree(node).index(),
            self.node_count(),
            batch_size as usize,
            concurrency,
        )
    }
}

impl<Node: Idx, D: DirectedGraph<Node>> OutDegreePartitionOp<Node> for D {
    /// Creates a greedy range-based out degree partition of the nodes.
    ///
    /// It is greedy in the sense that it goes through the node set only once
    /// and simply adds a new range to the result whenever the current range's
    /// nodes' out degrees sum up to at least the average node out degree.
    ///
    /// # Example
    ///
    /// ```
    /// # use graph::prelude::*;
    /// # use std::ops::Range;
    /// let graph: DirectedCsrGraph<u32> = GraphBuilder::new()
    ///     .edges(vec![(0, 1), (0, 2), (2, 1), (2, 3)])
    ///     .build();
    ///
    /// let partition: Vec<Range<u32>> = graph.out_degree_partition(2);
    ///
    /// assert_eq!(partition.len(), 2);
    /// assert_eq!(partition[0], 0..1);
    /// assert_eq!(partition[1], 1..4);
    /// ```
    fn out_degree_partition(&self, concurrency: usize) -> Vec<Range<Node>> {
        let batch_size = (self.edge_count().index() as f64 / concurrency as f64).ceil();
        greedy_node_map_partition(
            |node| self.out_degree(node).index(),
            self.node_count(),
            batch_size as usize,
            concurrency,
        )
    }
}

impl<Node: Idx, D: DirectedGraph<Node>> InDegreePartitionOp<Node> for D {
    /// Creates a greedy range-based in degree partition of the nodes.
    ///
    /// It is greedy in the sense that it goes through the node set only once
    /// and simply adds a new range to the result whenever the current range's
    /// nodes' in degrees sum up to at least the average node in degree.
    ///
    /// # Example
    ///
    /// ```
    /// # use graph::prelude::*;
    /// # use std::ops::Range;
    /// let graph: DirectedCsrGraph<u32> = GraphBuilder::new()
    ///     .edges(vec![(1, 0), (1, 2), (2, 0), (3, 2)])
    ///     .build();
    ///
    /// let partition: Vec<Range<u32>> = graph.in_degree_partition(2);
    ///
    /// assert_eq!(partition.len(), 2);
    /// assert_eq!(partition[0], 0..1);
    /// assert_eq!(partition[1], 1..4);
    /// ```
    fn in_degree_partition(&self, concurrency: usize) -> Vec<Range<Node>> {
        let batch_size = (self.edge_count().index() as f64 / concurrency as f64).ceil();
        greedy_node_map_partition(
            |node| self.in_degree(node).index(),
            self.node_count(),
            batch_size as usize,
            concurrency,
        )
    }
}

// Split input slice into a vector of partition.len() disjoint slices such that
// the slice at index i in the output vector has the same length as the range at
// index i in the input partition.
fn split_by_partition<'a, Node: Idx, T>(
    partition: &[Range<Node>],
    slice: &'a mut [T],
) -> Vec<&'a mut [T]> {
    debug_assert_eq!(
        partition
            .iter()
            .map(|r| r.end - r.start)
            .sum::<Node>()
            .index(),
        slice.len()
    );

    let mut splits = Vec::with_capacity(partition.len());

    let mut remainder = slice;
    let mut current_start = Node::zero();
    for range in partition.iter() {
        let next_end = range.end - current_start;
        current_start += next_end;

        let (left, right) = remainder.split_at_mut(next_end.index());

        splits.push(left);
        remainder = right;
    }

    splits
}

// Partition nodes 0..node_count().index() into at most max_batches ranges such
// that the sums of node_map(node) for each range are roughly equal. It does so
// greedily and therefore does not guarantee an optimally balanced range-based
// partition.
fn greedy_node_map_partition<Node, F>(
    node_map: F,
    node_count: Node,
    batch_size: usize,
    max_batches: usize,
) -> Vec<Range<Node>>
where
    F: Fn(Node) -> usize,
    Node: Idx,
{
    let mut partitions = Vec::with_capacity(max_batches);

    let mut partition_size = 0;
    let mut partition_start = Node::zero();
    let upper_bound = node_count - Node::new(1);

    for node in Node::zero()..node_count {
        partition_size += node_map(node);

        if (partitions.len() < max_batches - 1 && partition_size >= batch_size)
            || node == upper_bound
        {
            let partition_end = node + Node::new(1);
            partitions.push(partition_start..partition_end);
            partition_size = 0;
            partition_start = partition_end;
        }
    }

    partitions
}

fn relabel_by_degree<Node, G>(graph: &G) -> G
where
    Node: Idx,
    G: From<Csr<Node>> + UndirectedGraph<Node> + Sync,
{
    let start = Instant::now();
    let degree_node_pairs = sort_by_degree_desc(graph);
    info!("Relabel: sorted degree-node-pairs in {:?}", start.elapsed());

    let start = Instant::now();
    let (degrees, nodes) = unzip_degrees_and_nodes(degree_node_pairs);
    info!("Relabel: built degrees and id map in {:?}", start.elapsed());

    let start = Instant::now();
    let offsets = prefix_sum(degrees);
    let targets = relabel_targets(graph, nodes, &offsets);
    info!("Relabel: built and sorted targets in {:?}", start.elapsed());

    G::from(Csr::new(
        offsets.into_boxed_slice(),
        targets.into_boxed_slice(),
    ))
}

// Extracts (degree, node_id) pairs from the given graph and sorts them by
// degree descending.
fn sort_by_degree_desc<Node, G>(graph: &G) -> Vec<(Node, Node)>
where
    Node: Idx,
    G: From<Csr<Node>> + UndirectedGraph<Node> + Sync,
{
    let node_count = graph.node_count().index();
    let mut degree_node_pairs = Vec::with_capacity(node_count);

    (0..node_count)
        .into_par_iter()
        .map(Node::new)
        .map(|node_id| (graph.degree(node_id), node_id))
        .collect_into_vec(&mut degree_node_pairs);
    degree_node_pairs.par_sort_unstable_by(|left, right| left.cmp(right).reverse());

    degree_node_pairs
}

// Unzips (degree, node-id) pairs into `degrees` and `nodes`
//
// `degrees` maps a new node id to its degree.
// `nodes` maps the previous node id to the new node id.
fn unzip_degrees_and_nodes<Node: Idx>(
    degree_node_pairs: Vec<(Node, Node)>,
) -> (Vec<Node>, Vec<Node>) {
    let node_count = degree_node_pairs.len();
    let mut degrees = Vec::<Node>::with_capacity(node_count);
    let mut nodes = Vec::<Node>::with_capacity(node_count);
    let nodes_ptr = SharedMut::new(nodes.as_mut_ptr());

    (0..node_count)
        .into_par_iter()
        .map(|n| {
            let (degree, node) = degree_node_pairs[n];

            // SAFETY: node is the node_id from degree_node_pairs which is
            // created from 0..node_count -- the values are all distinct and we
            // will not write into the same location in parallel
            unsafe {
                nodes_ptr.add(node.index()).write(Node::new(n));
            }

            degree
        })
        .collect_into_vec(&mut degrees);

    // SAFETY: degree_node_pairs contains each value in 0..node_count once
    unsafe {
        nodes.set_len(node_count);
    }

    (degrees, nodes)
}

// Relabel target ids according to the given node mapping and offsets.
fn relabel_targets<Node, G>(graph: &G, nodes: Vec<Node>, offsets: &[Node]) -> Vec<Node>
where
    Node: Idx,
    G: From<Csr<Node>> + UndirectedGraph<Node> + Sync,
{
    let node_count = graph.node_count().index();
    let edge_count = offsets[node_count].index();
    let mut targets = Vec::<Node>::with_capacity(edge_count);
    let targets_ptr = SharedMut::new(targets.as_mut_ptr());

    (0..node_count)
        .into_par_iter()
        .map(Node::new)
        .for_each(|u| {
            let new_u = nodes[u.index()];
            let start_offset = offsets[new_u.index()].index();
            let mut end_offset = start_offset;

            for &v in graph.neighbors(u) {
                let new_v = nodes[v.index()];
                // SAFETY: a node u is processed by at most one thread. We write
                // into a non-overlapping range defined by the offsets for that
                // node. No two threads will write into the same range.
                unsafe {
                    targets_ptr.add(end_offset).write(new_v);
                }
                end_offset += 1;
            }

            // SAFETY: start_offset..end_offset is a non-overlapping range for
            // a node u which is processed by exactly one thread.
            unsafe {
                std::slice::from_raw_parts_mut(
                    targets_ptr.add(start_offset),
                    end_offset - start_offset,
                )
            }
            .sort_unstable();
        });

    // SAFETY: we inserted every relabeled target id of which there are edge_count many.
    unsafe {
        targets.set_len(edge_count);
    }

    targets
}

#[cfg(test)]
mod tests {
    use crate::{
        builder::GraphBuilder, graph::csr::UndirectedCsrGraph, graph_ops::unzip_degrees_and_nodes,
    };

    use super::*;

    #[test]
    fn split_by_partition_3_parts() {
        let partition = vec![0..2, 2..5, 5..10];
        let mut slice = (0..10).into_iter().collect::<Vec<_>>();
        let splits = split_by_partition(&partition, &mut slice);

        assert_eq!(splits.len(), partition.len());
        for (s, p) in splits.into_iter().zip(partition) {
            assert_eq!(s, p.into_iter().collect::<Vec<usize>>());
        }
    }

    #[test]
    fn split_by_partition_8_parts() {
        let partition = vec![0..1, 1..2, 2..3, 3..4, 4..6, 6..7, 7..8, 8..10];
        let mut slice = (0..10).into_iter().collect::<Vec<_>>();
        let splits = split_by_partition(&partition, &mut slice);

        assert_eq!(splits.len(), partition.len());
        for (s, p) in splits.into_iter().zip(partition) {
            assert_eq!(s, p.into_iter().collect::<Vec<usize>>());
        }
    }

    #[test]
    fn greedy_node_map_partition_1_part() {
        let partitions = greedy_node_map_partition::<usize, _>(|_| 1_usize, 10, 10, 99999);
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0], 0..10);
    }

    #[test]
    fn greedy_node_map_partition_2_parts() {
        let partitions = greedy_node_map_partition::<usize, _>(|x| x % 2_usize, 10, 4, 99999);
        assert_eq!(partitions.len(), 2);
        assert_eq!(partitions[0], 0..8);
        assert_eq!(partitions[1], 8..10);
    }

    #[test]
    fn greedy_node_map_partition_6_parts() {
        let partitions = greedy_node_map_partition::<usize, _>(|x| x as usize, 10, 6, 99999);
        assert_eq!(partitions.len(), 6);
        assert_eq!(partitions[0], 0..4);
        assert_eq!(partitions[1], 4..6);
        assert_eq!(partitions[2], 6..7);
        assert_eq!(partitions[3], 7..8);
        assert_eq!(partitions[4], 8..9);
        assert_eq!(partitions[5], 9..10);
    }

    #[test]
    fn greedy_node_map_partition_max_batches() {
        let partitions = greedy_node_map_partition::<usize, _>(|x| x as usize, 10, 6, 3);
        assert_eq!(partitions.len(), 3);
        assert_eq!(partitions[0], 0..4);
        assert_eq!(partitions[1], 4..6);
        assert_eq!(partitions[2], 6..10);
    }

    #[test]
    fn sort_by_degree_test() {
        let graph: UndirectedCsrGraph<_> = GraphBuilder::new()
            .edges::<u32, _>(vec![
                (0, 1),
                (1, 2),
                (1, 3),
                (2, 0),
                (2, 1),
                (2, 3),
                (3, 0),
                (3, 2),
            ])
            .build();

        assert_eq!(
            sort_by_degree_desc(&graph),
            vec![(5, 2), (4, 3), (4, 1), (3, 0)]
        );
    }

    #[test]
    fn unzip_degrees_and_nodes_test() {
        let degrees_and_nodes = vec![(5, 2), (4, 3), (4, 1), (3, 0)];

        let (degrees, nodes) = unzip_degrees_and_nodes::<u32>(degrees_and_nodes);

        assert_eq!(degrees, vec![5, 4, 4, 3]);
        assert_eq!(nodes, vec![3, 2, 0, 1]);
    }

    #[test]
    fn relabel_by_degree_test() {
        let graph: UndirectedCsrGraph<_> = GraphBuilder::new()
            .edges::<u32, _>(vec![
                (0, 1),
                (1, 2),
                (1, 3),
                (2, 0),
                (2, 1),
                (2, 3),
                (3, 0),
                (3, 2),
            ])
            .build();

        let relabeled_graph = graph.to_degree_ordered();

        assert_eq!(graph.node_count(), relabeled_graph.node_count());
        assert_eq!(graph.edge_count(), relabeled_graph.edge_count());

        // old -> new
        //   0 -> 3
        //   1 -> 2
        //   2 -> 0
        //   3 -> 1
        assert_eq!(relabeled_graph.degree(0), 5);
        assert_eq!(relabeled_graph.degree(1), 4);
        assert_eq!(relabeled_graph.degree(2), 4);
        assert_eq!(relabeled_graph.degree(3), 3);

        assert_eq!(relabeled_graph.neighbors(0), &[1, 1, 2, 2, 3]);
        assert_eq!(relabeled_graph.neighbors(1), &[0, 0, 2, 3]);
        assert_eq!(relabeled_graph.neighbors(2), &[0, 0, 1, 3]);
        assert_eq!(relabeled_graph.neighbors(3), &[0, 1, 2]);
    }
}
