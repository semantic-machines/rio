use crate::model::*;
use permutohedron::LexicalPermutation;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Hash)]
struct SubjectPredicate<'a> {
    subject: &'a OwnedNamedOrBlankNode,
    predicate: &'a OwnedNamedNode,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Hash)]
struct PredicateObject<'a> {
    predicate: &'a OwnedNamedNode,
    object: &'a OwnedTerm,
}

fn subject_predicates_for_object<'a>(
    dataset: &'a OwnedDataset,
    object: &'a OwnedTerm,
) -> impl Iterator<Item = SubjectPredicate<'a>> + 'a {
    dataset
        .triples_for_object(object)
        .map(|t| SubjectPredicate {
            subject: &t.subject,
            predicate: &t.predicate,
        })
}

fn predicate_objects_for_subject<'a>(
    dataset: &'a OwnedDataset,
    subject: &'a OwnedNamedOrBlankNode,
) -> impl Iterator<Item = PredicateObject<'a>> + 'a {
    dataset
        .triples_for_subject(subject)
        .map(|t| PredicateObject {
            predicate: &t.predicate,
            object: &t.object,
        })
}

fn hash_blank_nodes<'a>(
    bnodes: HashSet<&'a OwnedBlankNode>,
    dataset: &'a OwnedDataset,
) -> HashMap<u64, Vec<&'a OwnedBlankNode>> {
    let mut bnodes_by_hash = HashMap::default();

    // NB: we need to sort the triples to have the same hash
    for bnode in bnodes {
        let mut hasher = DefaultHasher::new();

        {
            let subject = OwnedNamedOrBlankNode::from(bnode.clone());
            let mut po_set: BTreeSet<PredicateObject<'_>> = BTreeSet::default();
            for po in predicate_objects_for_subject(dataset, &subject) {
                match &po.object {
                    OwnedTerm::BlankNode(_) => (),
                    _ => {
                        po_set.insert(po);
                    }
                }
            }
            for po in po_set {
                po.hash(&mut hasher);
            }
        }

        {
            let object = OwnedTerm::from(bnode.clone());
            let mut sp_set: BTreeSet<SubjectPredicate<'_>> = BTreeSet::default();
            for sp in subject_predicates_for_object(dataset, &object) {
                match &sp.subject {
                    OwnedNamedOrBlankNode::BlankNode(_) => (),
                    _ => {
                        sp_set.insert(sp);
                    }
                }
            }
            for sp in sp_set {
                sp.hash(&mut hasher);
            }
        }

        bnodes_by_hash
            .entry(hasher.finish())
            .or_insert_with(Vec::default)
            .push(bnode);
    }
    bnodes_by_hash
}

fn build_and_check_containment_from_hashes<'a>(
    hashes_to_see: &mut Vec<&u64>,
    a_bnodes_by_hash: &'a HashMap<u64, Vec<&'a OwnedBlankNode>>,
    b_bnodes_by_hash: &'a HashMap<u64, Vec<&'a OwnedBlankNode>>,
    a_to_b_mapping: &mut HashMap<&'a OwnedBlankNode, &'a OwnedBlankNode>,
    a: &OwnedDataset,
    b: &OwnedDataset,
) -> bool {
    let hash = match hashes_to_see.pop() {
        Some(h) => h,
        None => return check_is_contained(a_to_b_mapping, a, b),
    };

    const EMPTY_SLICE: &[&OwnedBlankNode] = &[];
    let a_nodes = a_bnodes_by_hash
        .get(hash)
        .map_or(EMPTY_SLICE, |v| v.as_slice());
    let b_nodes = b_bnodes_by_hash
        .get(hash)
        .map_or(EMPTY_SLICE, |v| v.as_slice());
    if a_nodes.len() != b_nodes.len() {
        return false;
    }
    if a_nodes.len() == 1 {
        // Avoid allocation for len == 1
        a_to_b_mapping.insert(a_nodes[0], b_nodes[0]);
        let result = build_and_check_containment_from_hashes(
            hashes_to_see,
            a_bnodes_by_hash,
            b_bnodes_by_hash,
            a_to_b_mapping,
            a,
            b,
        );
        a_to_b_mapping.remove(a_nodes[0]);
        hashes_to_see.push(hash);
        result
    } else {
        // We compute all the rotations of a_nodes and then zip it with b_nodes to have all the possible pairs (a,b)
        let mut a_nodes_rotated = a_nodes.to_vec();
        a_nodes_rotated.sort();
        loop {
            for (a_node, b_node) in a_nodes_rotated.iter().zip(b_nodes.iter()) {
                a_to_b_mapping.insert(a_node, b_node);
            }
            let result = if build_and_check_containment_from_hashes(
                hashes_to_see,
                a_bnodes_by_hash,
                b_bnodes_by_hash,
                a_to_b_mapping,
                a,
                b,
            ) {
                Some(true)
            } else if !a_nodes_rotated.next_permutation() {
                Some(false) // No more permutation
            } else {
                None //keep going
            };

            if let Some(result) = result {
                for a_node in &a_nodes_rotated {
                    a_to_b_mapping.remove(a_node);
                }
                hashes_to_see.push(hash);
                return result;
            }
        }
    }
}

fn check_is_contained<'a>(
    a_to_b_mapping: &mut HashMap<&'a OwnedBlankNode, &'a OwnedBlankNode>,
    a: &OwnedDataset,
    b: &OwnedDataset,
) -> bool {
    for t_a in a.iter() {
        let subject = if let OwnedNamedOrBlankNode::BlankNode(s_a) = &t_a.subject {
            a_to_b_mapping[s_a].clone().into()
        } else {
            t_a.subject.clone()
        };
        let predicate = t_a.predicate.clone();
        let object = if let OwnedTerm::BlankNode(o_a) = &t_a.object {
            a_to_b_mapping[o_a].clone().into()
        } else {
            t_a.object.clone()
        };
        let graph_name = if let Some(OwnedNamedOrBlankNode::BlankNode(g_a)) = &t_a.graph_name {
            Some(a_to_b_mapping[g_a].clone().into())
        } else {
            t_a.graph_name.clone()
        };
        if !b.contains(&OwnedQuad {
            subject,
            predicate,
            object,
            graph_name,
        }) {
            return false;
        }
    }

    true
}

fn dataset_blank_nodes(dataset: &OwnedDataset) -> HashSet<&OwnedBlankNode> {
    let mut blank_nodes = HashSet::default();
    for t in dataset.iter() {
        if let OwnedNamedOrBlankNode::BlankNode(subject) = &t.subject {
            blank_nodes.insert(subject);
        }
        if let OwnedTerm::BlankNode(object) = &t.object {
            blank_nodes.insert(object);
        }
        if let Some(OwnedNamedOrBlankNode::BlankNode(graph_name)) = &t.graph_name {
            blank_nodes.insert(graph_name);
        }
    }
    blank_nodes
}

pub fn are_datasets_isomorphic(a: &OwnedDataset, b: &OwnedDataset) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bnodes = dataset_blank_nodes(a);
    let a_bnodes_by_hash = hash_blank_nodes(a_bnodes, a);

    let b_bnodes = dataset_blank_nodes(b);
    let b_bnodes_by_hash = hash_blank_nodes(b_bnodes, b);

    // Hashes should have the same size everywhere
    if a_bnodes_by_hash.len() != b_bnodes_by_hash.len() {
        return false;
    }

    build_and_check_containment_from_hashes(
        &mut a_bnodes_by_hash.keys().collect(),
        &a_bnodes_by_hash,
        &b_bnodes_by_hash,
        &mut HashMap::default(),
        a,
        b,
    )
}
