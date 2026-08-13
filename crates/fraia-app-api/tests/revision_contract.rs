use fraia_revision::{ConversationId, ProjectId, RevisionId, SnapshotId};

#[test]
fn app_api_can_depend_on_revision_contract_without_reversing_core_dependency() {
    let project = ProjectId::from("project-a");
    let conversation = ConversationId::from("overall-framing");
    let revision = RevisionId::from("revision-1");
    let snapshot = SnapshotId::from("snapshot-1");

    assert_eq!(project.as_str(), "project-a");
    assert_eq!(conversation.as_str(), "overall-framing");
    assert_eq!(revision.as_str(), "revision-1");
    assert_eq!(snapshot.as_str(), "snapshot-1");
}
