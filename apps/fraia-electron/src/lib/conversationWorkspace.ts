import type { RenderScene, WorkbenchState } from './types';
import { projectDirOf } from './defaultProject';

export type ConversationTransportState = {
  projectId: string;
  conversationId: string;
  purpose: string;
  headRevisionId: string;
  headSnapshotId: string;
  messages?: string[];
  agentResponses?: ConversationAgentRespondResponse[];
  projectFacts?: ConversationProjectFacts;
};

export type ConversationProjectFacts = {
  name?: string;
  buildingType?: string;
  approximateLengthM?: number;
  approximateWidthM?: number;
  approximateHeightM?: number;
  objective?: string;
  constraints: string[];
  loadsAndAssumptions: string[];
  unknowns: string[];
};

export type ConversationRevisionProjection = {
  revisionId: string;
  snapshotId: string;
  parentRevisionId: string | null;
  author: 'agent' | 'manual' | 'system' | 'user';
  agentProvenance?: { provider: string; model: string; turnId: string } | null;
};

export type ConversationEvidenceProjection = {
  evidenceId: string;
  authoredSnapshotId: string;
  status: 'current' | 'stale' | 'failed' | 'unsupported';
};

export type ConversationAnalysisProjection = {
  evidenceId: string;
  snapshotId: string;
  status: 'success' | 'failed' | 'unsupported' | 'stale';
  summary: string;
};

export type ConversationArtefactProjection = {
  artefactId: string;
  kind: 'structural-preview' | 'analysis-result';
  sourceSnapshotId: string;
  scene: RenderScene;
};

export type ConversationProposalProjection = {
  proposalId: string;
  title: string;
  summary: string;
  parentRevisionId: string;
  proposedRevisionId: string;
  operation: ConversationStructuralOperation;
  operations?: ConversationStructuralOperation[];
  status: 'pending' | 'accepted' | 'rejected';
  analysed?: boolean;
  persisted?: boolean;
  assumptions?: string[];
  evidenceLimits?: string[];
};

export type ConversationAgentRespondResponse = {
  responseId: string;
  text: string;
  questions: string[];
  proposal?: {
    proposalId: string;
    proposedRevisionId: string;
    parentRevisionId: string;
    status?: 'pending' | 'accepted' | 'rejected';
    assumptions: string[];
    evidenceLimits: string[];
    operations: ConversationStructuralOperation[];
  };
  provider: string;
  model: string;
  reasoningEffort: string;
  catalogueRefreshedAt?: string;
  turnId: string;
};

export type ConversationComparisonProjection = {
  status: 'blocked' | 'available';
  summary: string;
  evidenceIds: string[];
  details?: {
    baselineSnapshotId: string;
    candidateSnapshotId: string;
    solverIdentity: string;
    inputIdentities: string[];
    resultIdentities: string[];
    maxUtilizations: number[];
  };
};

export type ConversationMessageProjection = {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  artefact?: ConversationArtefactProjection;
  proposal?: ConversationProposalProjection;
  proposals?: ConversationProposalProjection[];
  evidence?: ConversationEvidenceProjection;
  analysis?: ConversationAnalysisProjection;
};

export type ConversationWorkspaceProjection = {
  projectId: string;
  designId: string;
  /** Internal fraia-revision project scope. One design owns one revision DB. */
  revisionScopeId: string;
  projectDir: string;
  projectRootDir: string;
  conversationId: string;
  purpose: string;
  projectFacts: ConversationProjectFacts;
  head: ConversationRevisionProjection;
  messages: ConversationMessageProjection[];
  evidence: ConversationEvidenceProjection[];
  artefact: ConversationArtefactProjection;
  alternatives: ConversationProposalProjection[];
  comparison: ConversationComparisonProjection;
};

export type WorkingCopyProjection = {
  workingCopyId?: string;
  sourceRevisionId: string;
  sourceSnapshotId: string;
  scene: RenderScene;
  operationCount: number;
  operations?: ConversationStructuralOperation[];
  diffSummary?: string[];
  closed: boolean;
};

export type ConversationStructuralOperation = {
  kind: 'set_member_role';
  memberId: string;
  role: string;
} | {
  kind: 'move_node';
  nodeId: string;
  x: number;
  y: number;
  z: number;
} | {
  kind: 'add_node';
  id: string;
  x: number;
  y: number;
  z: number;
} | {
  kind: 'add_member';
  id: string;
  startNode: string;
  endNode: string;
  role: string;
  sectionId: string;
  materialId: string;
} | {
  kind: 'add_support';
  id: string;
  targetNode: string;
  ux: boolean;
  uy: boolean;
  uz: boolean;
  rx: boolean;
  ry: boolean;
  rz: boolean;
} | {
  kind: 'set_section';
  memberId: string;
  sectionId: string;
} | {
  kind: 'add_plate';
  id: string;
  boundaryNodes: string[];
  role: string;
  thicknessM: number;
  materialId: string;
  generatedFrom: string;
} | {
  kind: 'add_load';
  id: string;
  targetKind: 'node' | 'member' | 'plate';
  targetId: string;
  loadCaseId: string;
  directionX: number;
  directionY: number;
  directionZ: number;
  magnitude: number;
  unit: string;
} | {
  kind: 'add_release' | 'set_release';
  id: string;
  memberId: string;
  end: 'start' | 'end';
  ux: boolean;
  uy: boolean;
  uz: boolean;
  rx: boolean;
  ry: boolean;
  rz: boolean;
};

export type WorkingCopyOperation = ConversationStructuralOperation;

export type ConversationCreateRequest = {
  projectId: string;
  projectDir: string;
  conversationId: string;
  purpose: string;
  projectFacts: ConversationProjectFacts;
};

export type ConversationMessageRequest = {
  projectId: string;
  conversationId: string;
  purpose: string;
  message: string;
};

export type ConversationFactsUpdateRequest = {
  projectId: string;
  conversationId: string;
  projectFacts: ConversationProjectFacts;
};

export type ConversationAnalysisRequest = {
  projectId: string;
  conversationId: string;
  revisionId: string;
  evidenceId: string;
};

export type ConversationComparisonRequest = {
  projectId: string;
  conversationId: string;
  baselineEvidenceId: string;
  candidateEvidenceId: string;
};

export type ConversationForkRequest = {
  projectId: string;
  conversationId: string;
  purpose: string;
  fromRevisionId: string;
};

export type ConversationComparisonResponse = {
  solverIdentity: string;
  runtimeIdentity: string;
  settingsIdentity: string;
  settingsPayload: string;
  request: unknown;
  baseline: { evidenceId: string; authoredSnapshotId: string; resolvedSnapshotId: string; inputIdentity: string; resultIdentity: string; metrics: { maxUtilization: number; maxUxM: number; maxUyM: number; maxReactionN: number } };
  candidate: { evidenceId: string; authoredSnapshotId: string; resolvedSnapshotId: string; inputIdentity: string; resultIdentity: string; metrics: { maxUtilization: number; maxUxM: number; maxUyM: number; maxReactionN: number } };
};

export type ConversationEvidenceResponse = {
  evidenceId: string;
  authoredSnapshotId: string;
  stale: boolean;
  status?: 'success' | 'failed' | 'unsupported' | 'stale' | string;
  summary?: string;
  resolvedSnapshotId?: string;
  inputHash?: string;
  resultHash?: string;
  solverIdentity?: string;
  metrics?: {
    comboMetrics: Array<{ comboId: string; maxUtilization: number; maxUxM: number; maxUyM: number; maxReactionN: number }>;
    maxUtilization: number;
    maxUxM: number;
    maxUyM: number;
    maxReactionN: number;
  };
};

export type ConversationProposalRequest = {
  projectId: string;
  conversationId: string;
  proposalId: string;
  proposedRevisionId: string;
  parentRevisionId: string;
  provider: string;
  model: string;
  turnId: string;
  operation?: ConversationStructuralOperation;
  operations?: ConversationStructuralOperation[];
};

export async function respondConversationAgent(
  projection: ConversationWorkspaceProjection,
  text: string,
  turnId: string,
): Promise<{ projection: ConversationWorkspaceProjection; live: boolean }> {
  const respond = window.fraia?.conversationAgentRespond;
  if (!respond) return { projection, live: false };
  const shelf = await window.fraia?.listShelf?.({
    projectDir: projection.projectRootDir,
    designId: projection.designId,
  });
  const confirmedDesignReferenceIds = Object.values(shelf?.items ?? {})
    .filter((item) => item.confirmation?.confirmed === true)
    .map((item) => item.id)
    .sort();
  const interpretationList = await window.fraia?.listDrawingInterpretations?.({
    projectDir: projection.projectRootDir,
    designId: projection.designId,
  });
  const interpretationHead = interpretationList?.headRevisionId
    ? await window.fraia.inspectDrawingInterpretation({
      projectDir: projection.projectRootDir,
      designId: projection.designId,
      revisionId: interpretationList.headRevisionId,
    })
    : null;
  const confirmedInterpretationRevisionIds = interpretationHead
    && Object.values(interpretationHead.observations).some((observation) => observation.confirmation.status === 'confirmed' && observation.designGeometry)
    && !Object.values(interpretationHead.conflicts).some((conflict) => conflict.resolution.status === 'unresolved')
    ? [interpretationHead.revisionId]
    : [];
  const response = await respond({
    projectDir: projection.projectRootDir,
    packageProjectId: projection.projectId,
    projectId: projection.revisionScopeId,
    designId: projection.designId,
    conversationId: projection.conversationId,
    expectedHeadRevisionId: projection.head.revisionId,
    expectedSnapshotId: projection.head.snapshotId,
    text,
    shelfItemIds: confirmedDesignReferenceIds,
    drawingInterpretationRevisionIds: confirmedInterpretationRevisionIds,
    turnId,
  }) as ConversationAgentRespondResponse;
  const proposal = response.proposal && response.proposal.operations.length ? {
    proposalId: response.proposal.proposalId,
    title: 'Fraia proposal',
    summary: response.text,
    parentRevisionId: response.proposal.parentRevisionId,
    proposedRevisionId: response.proposal.proposedRevisionId,
    operation: response.proposal.operations[0],
    operations: response.proposal.operations,
    status: response.proposal.status ?? 'pending' as const,
    persisted: true,
    assumptions: response.proposal.assumptions,
    evidenceLimits: response.proposal.evidenceLimits,
  } : undefined;
  const assistant: ConversationMessageProjection = {
    id: `agent-response-${response.responseId}`,
    role: 'assistant',
    content: [response.text, ...response.questions].filter(Boolean).join('\n\n'),
    proposal,
  };
  return {
    live: true,
    projection: {
      ...projection,
      messages: [...projection.messages, assistant],
      alternatives: proposal ? [...projection.alternatives, proposal] : projection.alternatives,
    },
  };
}

export type ConversationProposalActionRequest = {
  projectId: string;
  proposalId: string;
  provider?: string;
  model?: string;
  turnId?: string;
};

export type ConversationWorkingCopyOpenRequest = {
  projectId: string;
  conversationId: string;
  revisionId: string;
};

export type ConversationWorkingCopyOperationRequest = {
  projectId: string;
  workingCopyId: string;
  operation: ConversationStructuralOperation;
};

export type ConversationWorkingCopyCommitRequest = {
  projectId: string;
  conversationId: string;
  workingCopyId: string;
  revisionId: string;
};

function sceneFromState(state: WorkbenchState): RenderScene {
  const scene = state.scene;
  const nodes = (scene?.nodes ?? []).flatMap((node) => {
    const position = Array.isArray(node.position) ? node.position : null;
    const x = node.x ?? (typeof position?.[0] === 'number' ? position[0] : undefined);
    const y = node.y ?? (typeof position?.[1] === 'number' ? position[1] : undefined);
    const z = node.z ?? (typeof position?.[2] === 'number' ? position[2] : undefined);
    return x != null && y != null && z != null ? [{ id: node.id, x, y, z }] : [];
  });
  const members = (scene?.members ?? []).flatMap((member) => {
    const start = member.start ?? member.startNode ?? member.start_node ?? member.i ?? member.node_i;
    const end = member.end ?? member.endNode ?? member.end_node ?? member.j ?? member.node_j;
    return start && end ? [{ id: member.id, role: member.role ?? 'member', start, end, sectionId: member.sectionId ?? member.section_id }] : [];
  });
  return {
    nodes,
    members,
    plates: scene?.plates ?? [],
    supports: scene?.supports ?? [],
    loads: scene?.loads ?? [],
    releases: scene?.releases ?? [],
    unitProfile: scene?.unitProfile ?? scene?.unit_profile,
  };
}

function asTextList(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.map((item) => String(item).trim()).filter(Boolean);
}

function numberValue(value: unknown): number | undefined {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (value && typeof value === 'object' && 'value' in value && typeof value.value === 'number' && Number.isFinite(value.value)) return value.value;
  return undefined;
}

export function projectFactsFromState(state: WorkbenchState): ConversationProjectFacts {
  const direct = state.projectFacts ?? state.project_facts ?? state.conversationFacts ?? state.conversation_facts ?? {};
  const draft = state.planningDraft ?? state.planning_draft ?? {};
  const intent = draft.projectIntent ?? draft.project_intent ?? {};
  const geometry = draft.geometryAndLoads ?? draft.geometry_and_loads ?? {};
  const dimensions = geometry.dimensions ?? {};
  const analysis = draft.analysisBrief ?? draft.analysis_brief ?? {};
  const brief: any = state.baseModelBrief ?? state.base_model_brief ?? {};
  const constraints = asTextList(direct.constraints ?? brief.confirmedIntent ?? brief.confirmed_intent);
  const loadsAndAssumptions = asTextList(direct.loadsAndAssumptions ?? direct.loads_and_assumptions);
  const unknowns = asTextList(direct.unknowns ?? brief.openQuestions ?? brief.open_questions);
  const gravity = numberValue(geometry.gravityLineLoad ?? geometry.gravity_line_load);
  const lateral = numberValue(geometry.lateralLoad ?? geometry.lateral_load);
  if (!loadsAndAssumptions.length) {
    if (gravity !== undefined) loadsAndAssumptions.push(`Gravity line load: ${gravity} N/m`);
    if (lateral !== undefined) loadsAndAssumptions.push(`Lateral load: ${lateral} N`);
    const summary = analysis.summaryGoals ?? analysis.summary_goals;
    if (typeof summary === 'string' && summary.trim()) loadsAndAssumptions.push(summary.trim());
  }
  return {
    name: direct.name ?? state.overview?.projectName ?? state.overview?.project_name ?? intent.name,
    buildingType: direct.buildingType ?? direct.building_type ?? intent.buildingType ?? intent.building_type,
    approximateLengthM: numberValue(direct.approximateLengthM ?? direct.approximate_length_m ?? geometry.span ?? dimensions.length),
    approximateWidthM: numberValue(direct.approximateWidthM ?? direct.approximate_width_m ?? geometry.width ?? geometry.depth ?? dimensions.width),
    approximateHeightM: numberValue(direct.approximateHeightM ?? direct.approximate_height_m ?? geometry.height ?? dimensions.height),
    objective: direct.objective ?? intent.objectivePriority ?? intent.objective_priority ?? analysis.summaryGoals ?? analysis.summary_goals,
    constraints,
    loadsAndAssumptions,
    unknowns,
  };
}

function cloneScene(scene: RenderScene): RenderScene {
  return {
    nodes: scene.nodes.map((node) => ({ ...node })),
    members: scene.members.map((member) => ({ ...member })),
    plates: scene.plates?.map((plate) => ({ ...plate, boundaryNodes: [...plate.boundaryNodes] })),
    supports: scene.supports.map((support) => ({ ...support })),
    loads: scene.loads.map((load) => ({ ...load })),
    releases: scene.releases?.map((release) => ({ ...release })),
    unitProfile: scene.unitProfile,
  };
}

function isFiniteCoordinate(value: number): boolean {
  return Number.isFinite(value);
}

export function describeConversationOperation(operation: ConversationStructuralOperation): string {
  switch (operation.kind) {
    case 'set_member_role': return `Set member ${operation.memberId} role to ${operation.role}`;
    case 'move_node': return `Move node ${operation.nodeId} to ${operation.x}, ${operation.y}, ${operation.z} m`;
    case 'add_node': return `Add node ${operation.id} at ${operation.x}, ${operation.y}, ${operation.z} m`;
    case 'add_member': return `Add ${operation.role} ${operation.id} from ${operation.startNode} to ${operation.endNode}`;
    case 'add_support': return `Add support ${operation.id} at node ${operation.targetNode}`;
    case 'set_section': return 'Set member ' + operation.memberId + ' section to ' + operation.sectionId;
    case 'add_plate': return 'Add ' + operation.role + ' plate ' + operation.id + ' (' + operation.thicknessM + ' m thick)';
    case 'add_load': return 'Add ' + operation.unit + ' ' + operation.targetKind + ' load ' + operation.id + ' to ' + operation.targetId;
    case 'add_release': return 'Add ' + operation.end + ' release ' + operation.id + ' to member ' + operation.memberId;
    case 'set_release': return 'Set ' + operation.end + ' release ' + operation.id + ' on member ' + operation.memberId;
  }
}

/** Apply one validated UI operation to an isolated render projection. */
export function applyConversationOperation(
  scene: RenderScene,
  operation: ConversationStructuralOperation,
): { scene: RenderScene; summary: string } | { error: string } {
  const next = cloneScene(scene);
  switch (operation.kind) {
    case 'set_member_role': {
      if (!operation.role.trim()) return { error: 'A member role is required.' };
      const member = next.members.find((item) => item.id === operation.memberId);
      if (!member) return { error: `Member ${operation.memberId} was not found.` };
      member.role = operation.role.trim();
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'move_node': {
      if (![operation.x, operation.y, operation.z].every(isFiniteCoordinate)) return { error: 'Node coordinates must be finite metre values.' };
      const node = next.nodes.find((item) => item.id === operation.nodeId);
      if (!node) return { error: `Node ${operation.nodeId} was not found.` };
      node.x = operation.x;
      node.y = operation.y;
      node.z = operation.z;
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'add_node': {
      if (!operation.id.trim()) return { error: 'A node id is required.' };
      if (![operation.x, operation.y, operation.z].every(isFiniteCoordinate)) return { error: 'Node coordinates must be finite metre values.' };
      if (next.nodes.some((item) => item.id === operation.id)) return { error: `Node ${operation.id} already exists.` };
      next.nodes.push({ id: operation.id.trim(), x: operation.x, y: operation.y, z: operation.z });
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'add_member': {
      if (!operation.id.trim() || !operation.role.trim()) return { error: 'A member id and role are required.' };
      if (operation.startNode === operation.endNode) return { error: 'A member needs two different nodes.' };
      if (!next.nodes.some((item) => item.id === operation.startNode) || !next.nodes.some((item) => item.id === operation.endNode)) return { error: 'Member endpoints must reference existing nodes.' };
      if (next.members.some((item) => item.id === operation.id)) return { error: `Member ${operation.id} already exists.` };
      next.members.push({ id: operation.id.trim(), role: operation.role.trim(), start: operation.startNode, end: operation.endNode });
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'add_support': {
      if (!operation.id.trim()) return { error: 'A support id is required.' };
      if (!next.nodes.some((item) => item.id === operation.targetNode)) return { error: `Node ${operation.targetNode} was not found.` };
      if (next.supports.some((item) => item.id === operation.id)) return { error: `Support ${operation.id} already exists.` };
      next.supports.push({ id: operation.id.trim(), targetNode: operation.targetNode, ux: operation.ux, uy: operation.uy, uz: operation.uz, rx: operation.rx, ry: operation.ry, rz: operation.rz });
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'set_section': {
      if (!operation.sectionId.trim()) return { error: 'A section id is required.' };
      const member = next.members.find((item) => item.id === operation.memberId);
      if (!member) return { error: 'Member ' + operation.memberId + ' was not found.' };
      member.sectionId = operation.sectionId.trim();
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'add_plate': {
      if (!operation.id.trim() || !operation.role.trim()) return { error: 'A plate id and role are required.' };
      if (!Number.isFinite(operation.thicknessM) || operation.thicknessM <= 0) return { error: 'Plate thickness must be a positive metre value.' };
      if (operation.boundaryNodes.length < 3 || operation.boundaryNodes.some((node) => !next.nodes.some((item) => item.id === node))) return { error: 'A plate needs at least three existing boundary nodes.' };
      if (next.plates?.some((plate) => plate.id === operation.id)) return { error: `Plate ${operation.id} already exists.` };
      next.plates = [...(next.plates ?? []), {
        id: operation.id.trim(),
        role: operation.role.trim(),
        boundaryNodes: [...operation.boundaryNodes],
        thicknessM: operation.thicknessM,
        materialId: operation.materialId.trim() || undefined,
        generatedFrom: operation.generatedFrom.trim() || undefined,
      }];
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'add_load': {
      if (!operation.id.trim() || !operation.targetId.trim()) return { error: 'A load id and target are required.' };
      if (![operation.directionX, operation.directionY, operation.directionZ, operation.magnitude].every(Number.isFinite)) return { error: 'Load direction and magnitude must be finite values.' };
      const targetExists = operation.targetKind === 'node'
        ? next.nodes.some((item) => item.id === operation.targetId)
        : operation.targetKind === 'member'
          ? next.members.some((item) => item.id === operation.targetId)
          : next.plates?.some((item) => item.id === operation.targetId) ?? false;
      if (!targetExists) return { error: 'Load target ' + operation.targetId + ' was not found.' };
      next.loads.push({ id: operation.id.trim(), kind: operation.targetKind, magnitude: operation.magnitude, x: operation.directionX, y: operation.directionY, z: operation.directionZ, targetNode: operation.targetKind === 'node' ? operation.targetId : undefined, targetMember: operation.targetKind === 'member' ? operation.targetId : undefined, targetPlate: operation.targetKind === 'plate' ? operation.targetId : undefined });
      return { scene: next, summary: describeConversationOperation(operation) };
    }
    case 'add_release':
    case 'set_release': {
      if (!operation.id.trim() || !operation.memberId.trim()) return { error: 'A release id and member are required.' };
      if (!next.members.some((item) => item.id === operation.memberId)) return { error: 'Member ' + operation.memberId + ' was not found.' };
      const release = { id: operation.id.trim(), memberId: operation.memberId, end: operation.end, ux: operation.ux, uy: operation.uy, uz: operation.uz, rx: operation.rx, ry: operation.ry, rz: operation.rz };
      next.releases = [...(next.releases ?? []).filter((item) => item.id !== release.id), release];
      return { scene: next, summary: describeConversationOperation(operation) };
    }
  }
}

export function applyConversationOperations(scene: RenderScene, operations: ConversationStructuralOperation[]): { scene: RenderScene; summaries: string[] } {
  return operations.reduce((current, operation) => {
    const applied = applyConversationOperation(current.scene, operation);
    if ('error' in applied) return current;
    return { scene: applied.scene, summaries: [...current.summaries, applied.summary] };
  }, { scene: cloneScene(scene), summaries: [] as string[] });
}

function transportOperation(operation: ConversationStructuralOperation): Record<string, unknown> {
  switch (operation.kind) {
    case 'set_member_role': return { kind: operation.kind, memberId: operation.memberId, role: operation.role };
    case 'move_node': return { kind: operation.kind, nodeId: operation.nodeId, x: operation.x, y: operation.y, z: operation.z };
    case 'add_node': return { kind: operation.kind, id: operation.id, x: operation.x, y: operation.y, z: operation.z };
    case 'add_member': return { kind: operation.kind, id: operation.id, startNode: operation.startNode, endNode: operation.endNode, role: operation.role, sectionId: operation.sectionId, materialId: operation.materialId };
    case 'add_support': return { kind: operation.kind, id: operation.id, targetNode: operation.targetNode, ux: operation.ux, uy: operation.uy, uz: operation.uz, rx: operation.rx, ry: operation.ry, rz: operation.rz };
    case 'set_section': return { kind: operation.kind, memberId: operation.memberId, sectionId: operation.sectionId };
    case 'add_plate': return { kind: operation.kind, id: operation.id, boundaryNodes: operation.boundaryNodes, role: operation.role, thicknessM: operation.thicknessM, materialId: operation.materialId, generatedFrom: operation.generatedFrom };
    case 'add_load': return { kind: operation.kind, id: operation.id, targetKind: operation.targetKind, targetId: operation.targetId, loadCaseId: operation.loadCaseId, directionX: operation.directionX, directionY: operation.directionY, directionZ: operation.directionZ, magnitude: operation.magnitude, unit: operation.unit };
    case 'add_release':
    case 'set_release': return { kind: operation.kind, id: operation.id, memberId: operation.memberId, end: operation.end, ux: operation.ux, uy: operation.uy, uz: operation.uz, rx: operation.rx, ry: operation.ry, rz: operation.rz };
  }
}

function markAcceptedProposal(
  message: ConversationMessageProjection,
  selectedProposalId: string,
): ConversationMessageProjection {
  const update = (proposal: ConversationProposalProjection): ConversationProposalProjection => ({
    ...proposal,
    status: proposal.proposalId === selectedProposalId ? 'accepted' : proposal.status,
  });
  return {
    ...message,
    proposal: message.proposal ? update(message.proposal) : undefined,
    proposals: message.proposals?.map(update),
  };
}

function markRejectedProposal(
  message: ConversationMessageProjection,
  selectedProposalId: string,
): ConversationMessageProjection {
  const update = (proposal: ConversationProposalProjection): ConversationProposalProjection => ({
    ...proposal,
    status: proposal.proposalId === selectedProposalId ? 'rejected' : proposal.status,
  });
  return {
    ...message,
    proposal: message.proposal ? update(message.proposal) : undefined,
    proposals: message.proposals?.map(update),
  };
}

function initialMessages(
  artefact: ConversationArtefactProjection,
  proposals: ConversationProposalProjection[],
): ConversationMessageProjection[] {
  const hasStructure = Boolean(
    artefact.scene.nodes.length
    || artefact.scene.members.length
    || artefact.scene.plates?.length,
  );
  if (!hasStructure) return [];
  return [
    {
      id: 'assistant-preview',
      role: 'assistant',
      content: 'This is the current structure. Inspect it, then review the proposed change below.',
      artefact,
      proposal: proposals[0],
      proposals,
    },
  ];
}

function agentAssistantMessages(state: WorkbenchState): ConversationMessageProjection[] {
  const agentState = state.agentState ?? state.agent_state;
  const session = agentState?.sessions?.find((item) => item.surface === 'pre_solve');
  return (session?.messages ?? [])
    .filter((message) => message.author === 'assistant' && message.text.trim())
    .map((message, index) => ({
      id: `agent-pre-solve-${message.createdAt ?? message.created_at ?? index}`,
      role: 'assistant' as const,
      content: message.text,
    }));
}

export function createConversationProjection(state: WorkbenchState): ConversationWorkspaceProjection {
  const projectDir = projectDirOf(state);
  const projectRootDir = state.overview?.projectRootDir ?? state.overview?.project_root_dir ?? projectDir;
  const projectId = state.overview?.projectId ?? state.overview?.project_id;
  const designId = state.overview?.designId ?? state.overview?.design_id ?? state.overview?.documentId ?? state.overview?.document_id;
  if (typeof projectId !== 'string' || !projectId || typeof designId !== 'string' || !designId) {
    throw new Error('Fraia requires stable project and design identity before starting a conversation.');
  }
  // Each design has its own durable revision database, so the canonical
  // overall-framing id remains stable when a legacy project is migrated.
  const conversationId = 'overall-framing';
  const revisionId = 'root-revision';
  const snapshotId = 'root-snapshot';
  const scene = sceneFromState(state);
  const projectFacts = projectFactsFromState(state);
  const artefact: ConversationArtefactProjection = {
    artefactId: 'current-structural-preview',
    kind: 'structural-preview',
    sourceSnapshotId: snapshotId,
    scene,
  };
  const alternatives: ConversationProposalProjection[] = [];
  const head: ConversationRevisionProjection = {
    revisionId,
    snapshotId,
    parentRevisionId: null,
    author: 'system',
  };
  return {
    projectId,
    designId,
    revisionScopeId: designId,
    projectDir,
    projectRootDir,
    conversationId,
    purpose: 'Overall framing',
    projectFacts,
    head,
    messages: [...initialMessages(artefact, alternatives), ...agentAssistantMessages(state)],
    evidence: [],
    artefact,
    alternatives,
    comparison: {
      status: 'blocked',
      summary: 'Analyse both directions to compare them.',
      evidenceIds: [],
    },
  };
}

function projectionFromTransport(
  current: ConversationWorkspaceProjection,
  response: ConversationTransportState,
): ConversationWorkspaceProjection {
  const persistedMessages = response.messages ?? [];
  const transportMessages: ConversationMessageProjection[] = persistedMessages.map((message, index) => ({
    id: `transport-message-${index}`,
    role: 'user' as const,
    content: message,
  }));
  const agentMessages: ConversationMessageProjection[] = (response.agentResponses ?? []).map((agent) => {
    const proposal = agent.proposal && agent.proposal.operations.length ? {
      proposalId: agent.proposal.proposalId,
      title: 'Fraia proposal',
      summary: agent.text,
      parentRevisionId: agent.proposal.parentRevisionId,
      proposedRevisionId: agent.proposal.proposedRevisionId,
      operation: agent.proposal.operations[0],
      operations: agent.proposal.operations,
      status: agent.proposal.status ?? 'pending' as const,
      persisted: true,
      assumptions: agent.proposal.assumptions,
      evidenceLimits: agent.proposal.evidenceLimits,
    } : undefined;
    return {
      id: `agent-response-${agent.responseId}`,
      role: 'assistant' as const,
      content: [agent.text, ...agent.questions].filter(Boolean).join('\n\n'),
      proposal,
    };
  });
  const acceptedOnRestart = current.head.revisionId === 'root-revision' && !response.headRevisionId.endsWith(':root');
  const resumableMessages = current.messages.filter((message) => message.proposal || message.artefact || message.id.startsWith('agent-pre-solve-'));
  const messages = persistedMessages.length || agentMessages.length
    ? [...transportMessages, ...agentMessages, ...resumableMessages]
    : current.messages;
  const messagesWithCurrentProposal = messages.map((message) => message.proposal
    ? {
      ...message,
      proposal: {
        ...message.proposal,
        status: acceptedOnRestart && message.proposal.proposalId === current.alternatives[0]?.proposalId ? 'accepted' as const : message.proposal.status,
        parentRevisionId: message.proposal.parentRevisionId === 'root-revision' ? response.headRevisionId : message.proposal.parentRevisionId,
        proposedRevisionId: message.proposal.proposedRevisionId === 'agent-revision-1' ? `agent-revision-${response.headRevisionId}` : message.proposal.proposedRevisionId,
      },
      proposals: message.proposals?.map((proposal) => ({
        ...proposal,
        status: acceptedOnRestart && proposal.proposalId === current.alternatives[0]?.proposalId ? 'accepted' as const : proposal.status,
        parentRevisionId: proposal.parentRevisionId === 'root-revision' ? response.headRevisionId : proposal.parentRevisionId,
        proposedRevisionId: proposal.proposedRevisionId.startsWith('agent-revision-root-revision-')
          ? proposal.proposedRevisionId.replace('agent-revision-root-revision-', `agent-revision-${response.headRevisionId}-`)
          : proposal.proposedRevisionId,
      })),
    }
    : message);
  const projectFacts = response.projectFacts
    ? {
      ...current.projectFacts,
      ...response.projectFacts,
      constraints: response.projectFacts.constraints ?? current.projectFacts.constraints,
      loadsAndAssumptions: response.projectFacts.loadsAndAssumptions ?? current.projectFacts.loadsAndAssumptions,
      unknowns: response.projectFacts.unknowns ?? current.projectFacts.unknowns,
    }
    : current.projectFacts;
  const acceptedOperations = messagesWithCurrentProposal.flatMap((message) => {
    const proposals = [...(message.proposals ?? []), ...(message.proposal ? [message.proposal] : [])];
    const accepted = proposals.find((proposal) => proposal.status === 'accepted');
    return accepted?.operations ?? [];
  });
  const restoredScene = acceptedOnRestart
    && !current.artefact.scene.nodes.length
    && !current.artefact.scene.members.length
    && acceptedOperations.length
    ? applyConversationOperations(current.artefact.scene, acceptedOperations).scene
    : current.artefact.scene;
  const restoredArtefact = { ...current.artefact, sourceSnapshotId: response.headSnapshotId, scene: restoredScene };
  const restoredMessages = acceptedOnRestart && !messagesWithCurrentProposal.some((message) => message.id.startsWith('restored-head-') || message.content.includes('Your current design was restored.'))
    ? [...messagesWithCurrentProposal, { id: `restored-head-${response.headRevisionId}`, role: 'system' as const, content: 'Your current design was restored.', artefact: restoredArtefact }]
    : messagesWithCurrentProposal;
  return {
    ...current,
    purpose: response.purpose,
    head: {
      ...current.head,
      revisionId: response.headRevisionId,
      snapshotId: response.headSnapshotId,
    },
    messages: restoredMessages,
    projectFacts,
    alternatives: current.alternatives.map((proposal) => ({
      ...proposal,
      parentRevisionId: response.headRevisionId,
      proposedRevisionId: `agent-revision-${response.headRevisionId}-${proposal.proposalId}`,
    })),
    comparison: current.comparison,
    artefact: restoredArtefact,
  };
}

export async function initializeConversation(
  projection: ConversationWorkspaceProjection,
): Promise<{ projection: ConversationWorkspaceProjection; live: boolean }> {
  const create = window.fraia?.conversationCreate as ((request: ConversationCreateRequest) => Promise<ConversationTransportState>) | undefined;
  if (!create) return { projection, live: false };
  try {
    const response = await create({
      projectId: projection.revisionScopeId,
      projectDir: projection.projectDir,
      conversationId: projection.conversationId,
      purpose: projection.purpose,
      projectFacts: projection.projectFacts,
    });
    return { projection: projectionFromTransport(projection, response), live: true };
  } catch (error) {
    console.warn('Conversation transport unavailable; using the typed local projection.', error);
    return { projection, live: false };
  }
}

export async function analyseConversationSnapshot(
  projection: ConversationWorkspaceProjection,
): Promise<{ analysis: ConversationAnalysisProjection; live: boolean }> {
  const evidenceId = `analysis-${projection.head.revisionId}`;
  const analyse = window.fraia?.conversationAnalyse as ((request: ConversationAnalysisRequest) => Promise<ConversationEvidenceResponse>) | undefined;
  if (!analyse) {
    return {
      analysis: {
        evidenceId,
        snapshotId: projection.head.snapshotId,
        status: 'unsupported',
        summary: 'Analysis transport is unavailable; no engineering evidence was recorded.',
      },
      live: false,
    };
  }
  try {
    const response = await analyse({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      revisionId: projection.head.revisionId,
      evidenceId,
    });
    return {
      analysis: {
        evidenceId: response.evidenceId,
        snapshotId: response.authoredSnapshotId,
        status: response.stale ? 'stale' : response.status === 'unsupported' ? 'unsupported' : response.status === 'failed' ? 'failed' : 'success',
        summary: response.stale ? 'The evidence is stale for the selected revision.' : response.summary || 'Snapshot-bound analysis evidence is current for this revision.',
      },
      live: true,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const unsupported = /unsupported|not available|not implemented/i.test(message);
    return {
      analysis: {
        evidenceId,
        snapshotId: projection.head.snapshotId,
        status: unsupported ? 'unsupported' : 'failed',
        summary: unsupported ? 'Analysis is not supported for this model yet.' : 'Analysis could not be completed for this revision.',
      },
      live: true,
    };
  }
}

export async function compareConversationEvidence(
  projection: ConversationWorkspaceProjection,
): Promise<{ comparison: ConversationComparisonProjection; live: boolean }> {
  const candidates = projection.evidence.filter((item) => item.status === 'current');
  if (candidates.length < 2) {
    return {
      comparison: {
        status: 'blocked',
        summary: 'Evidence-backed comparison is waiting for one analysed snapshot per candidate.',
        evidenceIds: candidates.map((item) => item.evidenceId),
      },
      live: false,
    };
  }
  const compare = window.fraia?.conversationCompare as ((request: ConversationComparisonRequest) => Promise<ConversationComparisonResponse>) | undefined;
  if (!compare) {
    return {
      comparison: {
        status: 'available',
        summary: 'Two current evidence records are available for comparison.',
        evidenceIds: candidates.slice(0, 2).map((item) => item.evidenceId),
      },
      live: false,
    };
  }
  try {
    const response = await compare({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      baselineEvidenceId: candidates[0].evidenceId,
      candidateEvidenceId: candidates[1].evidenceId,
    });
    return {
      comparison: {
        status: 'available',
        summary: 'Both candidates completed against the same deterministic solver and settings.',
        evidenceIds: [response.baseline.evidenceId, response.candidate.evidenceId],
        details: {
          baselineSnapshotId: response.baseline.authoredSnapshotId,
          candidateSnapshotId: response.candidate.authoredSnapshotId,
          solverIdentity: response.solverIdentity,
          inputIdentities: [response.baseline.inputIdentity, response.candidate.inputIdentity],
          resultIdentities: [response.baseline.resultIdentity, response.candidate.resultIdentity],
          maxUtilizations: [response.baseline.metrics.maxUtilization, response.candidate.metrics.maxUtilization],
        },
      },
      live: true,
    };
  } catch (error) {
    return {
      comparison: {
        status: 'blocked',
        summary: error instanceof Error ? error.message : String(error),
        evidenceIds: candidates.slice(0, 2).map((item) => item.evidenceId),
      },
      live: true,
    };
  }
}

export async function sendConversationMessage(
  projection: ConversationWorkspaceProjection,
  message: string,
): Promise<ConversationWorkspaceProjection> {
  const converse = window.fraia?.conversationConverse as ((request: ConversationMessageRequest) => Promise<ConversationTransportState>) | undefined;
  if (!converse) return projection;
  try {
    const response = await converse({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      purpose: projection.purpose,
      message,
    });
    const transported = projectionFromTransport(projection, response);
    return transported;
  } catch (error) {
    console.warn('Conversation message was kept in the local projection.', error);
    return projection;
  }
}

export async function updateConversationFacts(
  projection: ConversationWorkspaceProjection,
  projectFacts: ConversationProjectFacts,
): Promise<{ projection: ConversationWorkspaceProjection; live: boolean; error?: string }> {
  const update = window.fraia?.conversationFacts as ((request: ConversationFactsUpdateRequest) => Promise<ConversationTransportState>) | undefined;
  const localProjection = { ...projection, projectFacts };
  if (!update) {
    return {
      projection,
      live: false,
      error: 'The conversation transport is unavailable; the brief was not persisted.',
    };
  }
  try {
    const response = await update({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      projectFacts,
    });
    return { projection: projectionFromTransport(localProjection, response), live: true };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { projection, live: true, error: message };
  }
}

export async function acceptConversationProposal(
  projection: ConversationWorkspaceProjection,
  proposal: ConversationProposalProjection,
): Promise<{ projection: ConversationWorkspaceProjection; live: boolean; error?: string }> {
  const propose = window.fraia?.conversationPropose as ((request: ConversationProposalRequest) => Promise<unknown>) | undefined;
  const accept = window.fraia?.conversationAccept as ((request: ConversationProposalActionRequest) => Promise<ConversationRevisionProjection>) | undefined;
  const operations = proposal.operations?.length ? proposal.operations : [proposal.operation];
  const localScene = applyConversationOperations(projection.artefact.scene, operations).scene;
  if (!propose || !accept) {
    return {
      projection,
      live: false,
      error: 'The conversation transport is unavailable; no revision was created.',
    };
  }

  try {
    if (!proposal.persisted) await propose({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      proposalId: proposal.proposalId,
      proposedRevisionId: proposal.proposedRevisionId,
      parentRevisionId: proposal.parentRevisionId,
      provider: 'local-conversation-adapter',
      model: 'typed-projection',
      turnId: `turn-${proposal.proposalId}`,
      operations: operations.map(transportOperation) as ConversationProposalRequest['operations'],
      operation: undefined,
    });
    const revision = await accept({
      projectId: projection.revisionScopeId,
      proposalId: proposal.proposalId,
      provider: 'local-conversation-adapter',
      model: 'typed-projection',
      turnId: `turn-${proposal.proposalId}`,
    });
    return {
      projection: {
        ...projection,
        head: revision,
        artefact: { ...projection.artefact, sourceSnapshotId: revision.snapshotId, scene: localScene },
        messages: projection.messages.map((message) => message.proposal?.proposalId === proposal.proposalId || message.proposals?.some((item) => item.proposalId === proposal.proposalId)
          ? markAcceptedProposal(message, proposal.proposalId)
          : message),
      },
      live: true,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.warn('Agent proposal was not accepted by the live transport.', error);
    return { projection, live: true, error: message };
  }
}

export async function rejectConversationProposal(
  projection: ConversationWorkspaceProjection,
  proposal: ConversationProposalProjection,
): Promise<{ projection: ConversationWorkspaceProjection; live: boolean; error?: string }> {
  const reject = window.fraia?.conversationReject as ((request: ConversationProposalActionRequest) => Promise<unknown>) | undefined;
  const rejectedProjection = {
    ...projection,
    messages: projection.messages.map((message) => message.proposal?.proposalId === proposal.proposalId || message.proposals?.some((item) => item.proposalId === proposal.proposalId)
      ? markRejectedProposal(message, proposal.proposalId)
      : message),
  };
  if (!reject) return { projection: rejectedProjection, live: false };
  try {
    await reject({ projectId: projection.revisionScopeId, proposalId: proposal.proposalId });
    return { projection: rejectedProjection, live: true };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { projection, live: true, error: message };
  }
}

export async function analyseConversationAlternative(
  projection: ConversationWorkspaceProjection,
  proposal: ConversationProposalProjection,
): Promise<{ projection: ConversationWorkspaceProjection; live: boolean; error?: string }> {
  const fork = window.fraia?.conversationFork as ((request: ConversationForkRequest) => Promise<ConversationTransportState>) | undefined;
  const propose = window.fraia?.conversationPropose as ((request: ConversationProposalRequest) => Promise<unknown>) | undefined;
  const accept = window.fraia?.conversationAccept as ((request: ConversationProposalActionRequest) => Promise<ConversationRevisionProjection>) | undefined;
  const analyse = window.fraia?.conversationAnalyse as ((request: ConversationAnalysisRequest) => Promise<ConversationEvidenceResponse>) | undefined;
  if (!fork || !propose || !accept || !analyse) {
    return { projection, live: false, error: 'The conversation transport is unavailable; the alternative was not analysed.' };
  }
  const branchConversationId = `candidate-${proposal.proposalId}`;
  const branchProposalId = `${proposal.proposalId}-branch`;
  const branchRevisionId = `${proposal.proposedRevisionId}-branch`;
  try {
    await fork({
      projectId: projection.revisionScopeId,
      conversationId: branchConversationId,
      purpose: proposal.title,
      fromRevisionId: proposal.parentRevisionId,
    });
    const operations = proposal.operations?.length ? proposal.operations : [proposal.operation];
    await propose({
      projectId: projection.revisionScopeId,
      conversationId: branchConversationId,
      proposalId: branchProposalId,
      proposedRevisionId: branchRevisionId,
      parentRevisionId: proposal.parentRevisionId,
      provider: 'local-conversation-adapter',
      model: 'typed-projection',
      turnId: `turn-${branchProposalId}`,
      operations: operations.map(transportOperation) as ConversationProposalRequest['operations'],
      operation: undefined,
    });
    const revision = await accept({
      projectId: projection.revisionScopeId,
      proposalId: branchProposalId,
      provider: 'local-conversation-adapter',
      model: 'typed-projection',
      turnId: `turn-${branchProposalId}`,
    });
    const evidence = await analyse({
      projectId: projection.revisionScopeId,
      conversationId: branchConversationId,
      revisionId: revision.revisionId,
      evidenceId: `analysis-${revision.revisionId}`,
    });
    const candidateEvidence: ConversationEvidenceProjection = {
      evidenceId: evidence.evidenceId,
      authoredSnapshotId: evidence.authoredSnapshotId,
      status: evidence.stale ? 'stale' : evidence.status === 'failed' ? 'failed' : evidence.status === 'unsupported' ? 'unsupported' : 'current',
    };
    const messages = projection.messages.map((message) => message.proposals?.some((item) => item.proposalId === proposal.proposalId)
      ? { ...message, proposals: message.proposals?.map((item) => item.proposalId === proposal.proposalId ? { ...item, analysed: true } : item) }
      : message);
    return {
      projection: {
        ...projection,
        evidence: [...projection.evidence.filter((item) => item.evidenceId !== candidateEvidence.evidenceId), candidateEvidence],
        messages: [...messages, { id: `candidate-analysis-${candidateEvidence.evidenceId}`, role: 'assistant', content: `Candidate ${proposal.title} analysed in branch ${branchConversationId}. ${evidence.summary ?? ''}`, analysis: { evidenceId: candidateEvidence.evidenceId, snapshotId: candidateEvidence.authoredSnapshotId, status: candidateEvidence.status === 'current' ? 'success' : candidateEvidence.status, summary: evidence.summary ?? 'Candidate analysis completed.' } }],
      },
      live: true,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { projection, live: true, error: message };
  }
}

export async function openConversationWorkingCopy(
  projection: ConversationWorkspaceProjection,
): Promise<{ workingCopy: WorkingCopyProjection; live: boolean }> {
  const open = window.fraia?.conversationWorkingCopyOpen as ((request: ConversationWorkingCopyOpenRequest) => Promise<{
    workingCopyId: string;
    sourceRevisionId: string;
    sourceSnapshotId: string;
  }>) | undefined;
  const fallback: WorkingCopyProjection = {
    sourceRevisionId: projection.head.revisionId,
    sourceSnapshotId: projection.head.snapshotId,
    scene: cloneScene(projection.artefact.scene),
    operationCount: 0,
    operations: [],
    diffSummary: [],
    closed: false,
  };
  if (!open) return { workingCopy: fallback, live: false };
  try {
    const response = await open({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      revisionId: projection.head.revisionId,
    });
    return {
      workingCopy: { ...fallback, workingCopyId: response.workingCopyId, sourceRevisionId: response.sourceRevisionId, sourceSnapshotId: response.sourceSnapshotId },
      live: true,
    };
  } catch (error) {
    console.warn('Working-copy transport unavailable; using the typed local handoff.', error);
    return { workingCopy: fallback, live: false };
  }
}

export async function applyConversationWorkingCopyOperation(
  projection: ConversationWorkspaceProjection,
  workingCopy: WorkingCopyProjection,
  operation: WorkingCopyOperation,
): Promise<boolean> {
  const apply = window.fraia?.conversationWorkingCopyApply as ((request: ConversationWorkingCopyOperationRequest) => Promise<unknown>) | undefined;
  // A missing transport is the deliberate local projection fallback used by
  // component tests and offline startup; the caller still owns the isolated
  // render copy and can apply the same typed operation locally.
  if (!apply || !workingCopy.workingCopyId) return true;
  try {
    await apply({
      projectId: projection.revisionScopeId,
      workingCopyId: workingCopy.workingCopyId,
      operation: transportOperation(operation) as ConversationWorkingCopyOperationRequest['operation'],
    });
    return true;
  } catch (error) {
    console.warn('Working-copy operation kept in the local projection.', error);
    return false;
  }
}

export async function commitConversationWorkingCopy(
  projection: ConversationWorkspaceProjection,
  workingCopy: WorkingCopyProjection,
): Promise<{ revision: ConversationRevisionProjection | null; live: boolean; error?: string }> {
  const commit = window.fraia?.conversationWorkingCopyCommit as ((request: ConversationWorkingCopyCommitRequest) => Promise<ConversationRevisionProjection>) | undefined;
  if (!commit || !workingCopy.workingCopyId) {
    return {
      revision: null,
      live: false,
      error: 'The working-copy transport is unavailable; no manual revision was created.',
    };
  }
  try {
    const revision = await commit({
      projectId: projection.revisionScopeId,
      conversationId: projection.conversationId,
      workingCopyId: workingCopy.workingCopyId,
      revisionId: `manual-revision-${projection.head.revisionId}`,
    });
    return { revision, live: true };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.warn('Working-copy commit could not be completed.', error);
    return { revision: null, live: false, error: message };
  }
}
