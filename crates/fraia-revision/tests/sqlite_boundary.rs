use fraia_revision::snapshot::{ModelSnapshot, Sha256SnapshotIdentityDeriver};
use fraia_revision::sqlite::{
    SqliteRepositoryError, SqliteRevisionRepository, StoredArtefact, StoredConversation,
    StoredEvidence, StoredProjectRoot, StoredRevision, StoredSnapshot,
};
use fraia_revision::{
    ArtefactId, ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId,
    SnapshotIdentityDeriver, root_fixture,
};
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

fn stored_bytes(bytes: &[u8]) -> StoredSnapshot {
    StoredSnapshot {
        id: Sha256SnapshotIdentityDeriver
            .derive_snapshot_id(bytes)
            .unwrap(),
        format_version: "test-v1".into(),
        canonical_bytes: bytes.to_vec(),
    }
}

fn stored_model(snapshot: &ModelSnapshot) -> StoredSnapshot {
    StoredSnapshot {
        id: snapshot.id().clone(),
        format_version: snapshot.canonical_format_version().as_str().into(),
        canonical_bytes: snapshot.canonical_bytes().to_vec(),
    }
}

fn project_root(
    project_id: &str,
    conversation_id: &str,
    revision_id: &str,
    snapshot: StoredSnapshot,
) -> StoredProjectRoot {
    let project_id = ProjectId::from(project_id);
    let conversation_id = ConversationId::from(conversation_id);
    let revision_id = RevisionId::from(revision_id);
    StoredProjectRoot {
        project_id: project_id.clone(),
        root_conversation: StoredConversation {
            id: conversation_id.clone(),
            project_id,
            purpose: "Overall framing".into(),
            origin_json: "{\"kind\":\"root\"}".into(),
            head_revision_id: revision_id.clone(),
        },
        root_revision: StoredRevision {
            id: revision_id,
            snapshot_id: snapshot.id.clone(),
            parent_revision_id: None,
            conversation_id,
            metadata_json: "{\"operation\":\"root\"}".into(),
        },
        root_snapshot: snapshot,
    }
}

fn child_revision(
    id: &str,
    snapshot: &StoredSnapshot,
    parent_revision_id: &str,
    conversation_id: &str,
) -> StoredRevision {
    StoredRevision {
        id: RevisionId::from(id),
        snapshot_id: snapshot.id.clone(),
        parent_revision_id: Some(RevisionId::from(parent_revision_id)),
        conversation_id: ConversationId::from(conversation_id),
        metadata_json: "{\"provider\":\"test-provider\",\"turn\":\"turn-1\"}".into(),
    }
}

#[test]
fn restart_hydrates_project_lineage_typed_snapshot_evidence_artefact_and_provenance() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("restart.sqlite");
    let fixture = root_fixture();
    let root_snapshot = ModelSnapshot::capture(fixture.model.clone()).unwrap();
    let mut child_model = fixture.model;
    child_model.nodes[0].x = 0.5;
    let child_snapshot = ModelSnapshot::capture(child_model).unwrap();
    let root_snapshot = stored_model(&root_snapshot);
    let child_snapshot = stored_model(&child_snapshot);

    {
        let mut repository = SqliteRevisionRepository::open(&path).unwrap();
        repository
            .create_project(project_root(
                "project-a",
                "overall",
                "r0",
                root_snapshot.clone(),
            ))
            .unwrap();
        repository.insert_snapshot(&child_snapshot).unwrap();
        repository
            .append_revision(
                &child_revision("r1", &child_snapshot, "r0", "overall"),
                &RevisionId::from("r0"),
            )
            .unwrap();
        repository
            .create_conversation(&StoredConversation {
                id: ConversationId::from("alternative"),
                project_id: ProjectId::from("project-a"),
                purpose: "Alternative framing".into(),
                origin_json: "{\"kind\":\"fork\"}".into(),
                head_revision_id: RevisionId::from("r0"),
            })
            .unwrap();
        repository
            .attach_evidence(&StoredEvidence {
                id: EvidenceId::from("e1"),
                authored_snapshot_id: root_snapshot.id.clone(),
                resolved_snapshot_id: Some(child_snapshot.id.clone()),
                manifest_json: "{\"solver\":\"calculix:test\",\"turn\":\"turn-1\"}".into(),
                blob_ref: Some("blobs/e1".into()),
            })
            .unwrap();
        repository
            .attach_artefact(&StoredArtefact {
                id: ArtefactId::from("a1"),
                kind: "preview".into(),
                source_snapshot_id: root_snapshot.id.clone(),
                source_evidence_id: Some(EvidenceId::from("e1")),
                manifest_json: "{\"renderer\":\"v1\"}".into(),
                blob_ref: Some("blobs/a1".into()),
            })
            .unwrap();
    }

    let repository = SqliteRevisionRepository::open(&path).unwrap();
    let root = repository
        .project_root(&ProjectId::from("project-a"))
        .unwrap();
    assert_eq!(root.root_revision.id, RevisionId::from("r0"));
    assert_eq!(
        root.root_conversation.head_revision_id,
        RevisionId::from("r1")
    );
    assert_eq!(root.root_snapshot, root_snapshot);
    assert_eq!(
        repository
            .project_conversations(&ProjectId::from("project-a"))
            .unwrap()
            .iter()
            .map(|conversation| conversation.id.as_str())
            .collect::<Vec<_>>(),
        vec!["alternative", "overall"]
    );
    assert_eq!(
        repository
            .project_revisions(&ProjectId::from("project-a"))
            .unwrap()
            .iter()
            .map(|revision| revision.id.as_str())
            .collect::<Vec<_>>(),
        vec!["r0", "r1"]
    );
    assert_eq!(
        repository
            .revision(&RevisionId::from("r1"))
            .unwrap()
            .metadata_json,
        "{\"provider\":\"test-provider\",\"turn\":\"turn-1\"}"
    );
    assert_eq!(
        repository
            .hydrate_snapshot(&child_snapshot.id)
            .unwrap()
            .model()
            .nodes[0]
            .x,
        0.5
    );
    assert_eq!(
        repository
            .evidence(&EvidenceId::from("e1"))
            .unwrap()
            .manifest_json,
        "{\"solver\":\"calculix:test\",\"turn\":\"turn-1\"}"
    );
    assert_eq!(
        repository
            .artefact(&ArtefactId::from("a1"))
            .unwrap()
            .manifest_json,
        "{\"renderer\":\"v1\"}"
    );
    assert!(matches!(
        repository.artefact(&ArtefactId::from("missing")),
        Err(SqliteRepositoryError::UnknownArtefact(_))
    ));
}

#[test]
fn atomic_append_rolls_back_candidate_snapshot_on_expected_head_conflict() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("atomic-cas.sqlite");
    let root_snapshot = stored_bytes(b"root");
    let winner_snapshot = stored_bytes(b"winner");
    let loser_snapshot = stored_bytes(b"loser");
    {
        let mut repository = SqliteRevisionRepository::open(&path).unwrap();
        repository
            .create_project(project_root(
                "project-a",
                "overall",
                "r0",
                root_snapshot.clone(),
            ))
            .unwrap();
        repository
            .append_revision_with_snapshot(
                &child_revision("r1", &winner_snapshot, "r0", "overall"),
                &winner_snapshot,
                &RevisionId::from("r0"),
            )
            .unwrap();
        let error = repository
            .append_revision_with_snapshot(
                &child_revision("r2", &loser_snapshot, "r0", "overall"),
                &loser_snapshot,
                &RevisionId::from("r0"),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            SqliteRepositoryError::ExpectedHeadConflict { .. }
        ));
    }

    let repository = SqliteRevisionRepository::open(&path).unwrap();
    assert!(repository.revision(&RevisionId::from("r2")).is_err());
    assert!(repository.snapshot(&loser_snapshot.id).is_err());
    assert_eq!(
        repository
            .conversation(&ConversationId::from("overall"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("r1")
    );
}

#[test]
fn concurrent_atomic_appends_have_one_success_and_one_expected_head_conflict() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("concurrent-cas.sqlite");
    {
        let mut repository = SqliteRevisionRepository::open(&path).unwrap();
        repository
            .create_project(project_root(
                "project-a",
                "overall",
                "r0",
                stored_bytes(b"root"),
            ))
            .unwrap();
    }

    let barrier = Arc::new(Barrier::new(2));
    let first_path = path.clone();
    let first_barrier = Arc::clone(&barrier);
    let first = thread::spawn(move || {
        let mut repository = SqliteRevisionRepository::open(first_path).unwrap();
        let snapshot = stored_bytes(b"first");
        first_barrier.wait();
        repository.append_revision_with_snapshot(
            &child_revision("r1", &snapshot, "r0", "overall"),
            &snapshot,
            &RevisionId::from("r0"),
        )
    });
    let second_path = path.clone();
    let second_barrier = Arc::clone(&barrier);
    let second = thread::spawn(move || {
        let mut repository = SqliteRevisionRepository::open(second_path).unwrap();
        let snapshot = stored_bytes(b"second");
        second_barrier.wait();
        repository.append_revision_with_snapshot(
            &child_revision("r2", &snapshot, "r0", "overall"),
            &snapshot,
            &RevisionId::from("r0"),
        )
    });

    let results = [first.join().unwrap(), second.join().unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(SqliteRepositoryError::ExpectedHeadConflict { .. })
            ))
            .count(),
        1
    );
}

#[test]
fn online_backup_captures_active_wal_and_reopens_with_exact_lineage() {
    let directory = tempdir().unwrap();
    let source_path = directory.path().join("live.sqlite");
    let target_path = directory.path().join("relocated.sqlite");
    let fixture = root_fixture();
    let root_snapshot = ModelSnapshot::capture(fixture.model.clone()).unwrap();
    let mut child_model = fixture.model;
    child_model.nodes[0].x = 0.75;
    let child_snapshot = ModelSnapshot::capture(child_model).unwrap();
    let root_snapshot = stored_model(&root_snapshot);
    let child_snapshot = stored_model(&child_snapshot);

    let mut source = SqliteRevisionRepository::open(&source_path).unwrap();
    source
        .create_project(project_root(
            "project-a",
            "overall",
            "r0",
            root_snapshot.clone(),
        ))
        .unwrap();
    source
        .append_revision_with_snapshot(
            &child_revision("r1", &child_snapshot, "r0", "overall"),
            &child_snapshot,
            &RevisionId::from("r0"),
        )
        .unwrap();

    let wal_path = std::path::PathBuf::from(format!("{}-wal", source_path.display()));
    assert!(wal_path.exists(), "source must remain open in WAL mode");
    assert!(std::fs::metadata(&wal_path).unwrap().len() > 0);

    source.backup_to_path(&target_path).unwrap();
    assert!(target_path.exists());

    let relocated = SqliteRevisionRepository::open(&target_path).unwrap();
    assert_eq!(
        relocated
            .conversation(&ConversationId::from("overall"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("r1")
    );
    assert_eq!(
        relocated
            .history(&ConversationId::from("overall"))
            .unwrap()
            .iter()
            .map(|revision| revision.id.as_str())
            .collect::<Vec<_>>(),
        vec!["r1", "r0"]
    );
    assert_eq!(
        relocated
            .hydrate_snapshot(&child_snapshot.id)
            .unwrap()
            .model()
            .nodes[0]
            .x,
        0.75
    );
    assert_eq!(
        relocated
            .project_root(&ProjectId::from("project-a"))
            .unwrap()
            .root_snapshot,
        root_snapshot
    );

    let error = source.backup_to_path(&target_path).unwrap_err();
    assert!(matches!(
        error,
        SqliteRepositoryError::BackupTargetExists(path) if path == target_path
    ));
}

#[test]
fn conversations_cannot_point_at_a_revision_from_another_project() {
    let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
    repository
        .create_project(project_root(
            "project-a",
            "overall-a",
            "a0",
            stored_bytes(b"a-root"),
        ))
        .unwrap();
    repository
        .create_project(project_root(
            "project-b",
            "overall-b",
            "b0",
            stored_bytes(b"b-root"),
        ))
        .unwrap();

    let error = repository
        .create_conversation(&StoredConversation {
            id: ConversationId::from("cross-project"),
            project_id: ProjectId::from("project-b"),
            purpose: "invalid cross-project head".into(),
            origin_json: "{}".into(),
            head_revision_id: RevisionId::from("a0"),
        })
        .unwrap_err();
    assert!(matches!(
        error,
        SqliteRepositoryError::RevisionNotInProject { .. }
    ));
}

#[test]
fn persisted_snapshot_identity_is_immutable_and_validated() {
    let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
    let snapshot = stored_bytes(b"content");
    repository.insert_snapshot(&snapshot).unwrap();

    let mut changed_format = snapshot.clone();
    changed_format.format_version = "other-v1".into();
    assert!(matches!(
        repository.insert_snapshot(&changed_format),
        Err(SqliteRepositoryError::ImmutableSnapshotConflict(_))
    ));

    let invalid = StoredSnapshot {
        id: SnapshotId::from("sha256:not-content"),
        ..snapshot
    };
    assert!(matches!(
        repository.insert_snapshot(&invalid),
        Err(SqliteRepositoryError::InvalidSnapshotIdentity(_))
    ));
}
