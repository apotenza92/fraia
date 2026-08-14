export type Confirmation =
  | { status: 'unconfirmed' }
  | { status: 'confirmed'; confirmedBy: string; confirmedAt: string }
  | { status: 'rejected'; rejectedBy: string; rejectedAt: string; reason: string };

export type DrawingObservation = {
  id: string;
  shelfItemId: string;
  sourceId: string;
  sourceSha256: string;
  sourceLocator: Record<string, unknown> & { locatorKind: string };
  viewRole: 'plan' | 'elevation' | 'section' | 'detail' | 'schedule' | 'reference';
  sourceGeometry: Record<string, unknown> & { sourceGeometryKind: string };
  designGeometry?: Record<string, unknown> & { designGeometryKind: string };
  extraction: {
    method: string;
    producer: string;
    producerVersion: string;
    confidence: number;
    uncertainty?: Array<{ kind: string; message: string }>;
  };
  confirmation: Confirmation;
  featureKind: string;
  [key: string]: unknown;
};

export type DrawingConflict = {
  id: string;
  observationIds: string[];
  conflictKind: string;
  message: string;
  resolution: { status: 'unresolved' } | { status: 'resolved'; resolution: string; resolvedBy: string; resolvedAt: string };
};

export type DrawingInterpretation = {
  schemaVersion: string;
  projectId: string;
  designId: string;
  revisionId: string;
  parentRevisionId?: string;
  createdAt: string;
  method: string;
  observations: Record<string, DrawingObservation>;
  correspondences: Record<string, unknown>;
  alignmentTransforms: Record<string, unknown>;
  conflicts: Record<string, DrawingConflict>;
};

export type DrawingInterpretationList = {
  projectId: string;
  designId: string;
  headRevisionId?: string;
  revisions: Array<{
    revisionId: string;
    parentRevisionId?: string;
    createdAt: string;
    observationCount: number;
    unresolvedConflictCount: number;
  }>;
};

export type DesignRunStatus = 'completed' | 'failed' | 'unsupported';
export type DesignRunDiagnostic = { severity: 'information' | 'warning' | 'error'; code: string; message: string };
export type DesignRunSummary = {
  runId: string;
  runKind: string;
  createdAt?: string;
  created_at?: string;
  status: DesignRunStatus;
  authoredRevisionId: string;
  authoredSnapshotId: string;
  parentRunId?: string;
};
export type DesignRunList = { projectId: string; designId: string; runs: DesignRunSummary[]; legacyRuns: Array<{ directoryName: string }> };
export type DesignRunStatusProjection = {
  runId: string;
  status: DesignRunStatus;
  staleness: 'current' | 'stale_descendant' | 'stale_dependency' | 'unrelated';
  interpretationDependencies?: { revisionIds: string[]; inferenceIds: string[] };
  stalenessReasons?: Array<{ code: string; message: string; interpretationRevisionId?: string; inferenceId?: string; currentInterpretationRevisionId?: string }>;
  authoredRevisionId: string;
  authoredSnapshotId: string;
  resolvedSnapshotId?: string;
  solverIdentity: string;
  runtimeIdentity: string;
  settingsIdentity: string;
  diagnostics: DesignRunDiagnostic[];
};
export type DesignRunManifest = DesignRunSummary & {
  schemaVersion: string;
  projectId: string;
  designId: string;
  actor: { actorType: string; actorId: string };
  resolvedSnapshotId?: string;
  requestIdentity: string;
  request: unknown;
  settingsIdentity: string;
  settings: unknown;
  solverIdentity: string;
  runtimeIdentity: string;
  inputIdentity?: string;
  resultIdentity?: string;
  diagnostics: DesignRunDiagnostic[];
  metrics?: unknown;
  attachments: Array<{ name: string; role: string; mediaType: string; sha256: string; byteSize: number }>;
};
export type InspectedDesignRun =
  | { format: 'canonical'; manifest: DesignRunManifest }
  | { format: 'legacy'; directoryName: string; runJson?: unknown; files: string[] };
