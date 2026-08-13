use fraia_core::StructuralNode;
use fraia_revision::patch::{Length, NodeInput, Position, StructuralOperation, StructuralPatch};
use fraia_revision::repository::{InMemoryRevisionRepository, ProposalId};
use fraia_revision::{RevisionId, root_fixture};

#[test]
fn accepted_patch_creates_a_child_without_mutating_the_parent_snapshot() {
    let fixture = root_fixture();
    let root_revision = fixture.root_revision_id.clone();
    let root_conversation = fixture.conversation_id.clone();
    let mut repository = InMemoryRevisionRepository::create(
        fixture.project_id,
        root_conversation.clone(),
        "overall framing",
        root_revision.clone(),
        fixture.model,
    )
    .unwrap();
    let parent_snapshot = repository
        .snapshot(repository.revision(&root_revision).unwrap().snapshot_id())
        .unwrap()
        .clone();
    repository
        .create_proposal(
            ProposalId::from("add-inspection-node"),
            root_conversation,
            root_revision.clone(),
            RevisionId::from("child"),
            StructuralPatch {
                operations: vec![StructuralOperation::AddNode(NodeInput {
                    id: "inspection-node".into(),
                    position: Position {
                        x: Length::meters(10.0),
                        y: Length::meters(0.0),
                        z: Length::meters(0.0),
                    },
                })],
            },
        )
        .unwrap();
    repository
        .accept_proposal(&ProposalId::from("add-inspection-node"))
        .unwrap();

    let root_after = repository
        .snapshot(repository.revision(&root_revision).unwrap().snapshot_id())
        .unwrap();
    assert_eq!(parent_snapshot.id(), root_after.id());
    assert_eq!(
        parent_snapshot.canonical_bytes(),
        root_after.canonical_bytes()
    );
    assert!(
        root_after
            .model()
            .nodes
            .iter()
            .all(|node: &StructuralNode| node.id != "inspection-node")
    );
    assert!(
        repository
            .snapshot(
                repository
                    .revision(&RevisionId::from("child"))
                    .unwrap()
                    .snapshot_id()
            )
            .unwrap()
            .model()
            .node_by_id("inspection-node")
            .is_some()
    );
}
