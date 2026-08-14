import { useEffect, useRef, useState } from 'react';
import { Check, History, Maximize2, MessageSquareText, PencilLine, Send, X } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { Field, FieldDescription, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupTextarea } from '@/components/ui/input-group';
import { Marker, MarkerContent, MarkerIcon } from '@/components/ui/marker';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';
import {
  ChatTranscript,
  ChatTranscriptActivity,
  ChatTranscriptCancel,
  ChatTranscriptMessage,
  ChatTranscriptPanel,
} from '@/components/chat/ChatTranscript';
import { Viewport3D } from '@/components/viewport/Viewport3D';
import { cn } from '@/lib/utils';
import { AnalysisHistorySheet } from './AnalysisHistorySheet';
import type { WorkbenchState } from '@/lib/types';
type AnalysisAttemptResponse = Awaited<ReturnType<typeof window.fraia.analysisAttemptStatus>>;

function analysisDiagnosticMessage(diagnostic: string) {
  if (diagnostic.startsWith('analysis.cancelled:')) return 'Analysis cancelled. The accepted design was not changed.';
  if (diagnostic.startsWith('analysis.test-forced-failure:')) return 'Analysis failed before Fraia could publish a result. Retry starts a new attempt.';
  return diagnostic;
}

function friendlyTransportMessage(detail: string) {
  const normalized = detail.toLowerCase();
  if (/timed?\s*out|timeout|too long/.test(normalized)) return 'Fraia took too long to respond. Try again.';
  if (/schema|validation|unknown (section|reference)|invalid design proposal/.test(normalized)) return 'Fraia could not prepare a valid design proposal. Try again or clarify your request.';
  if (/connect|unavailable|contact|network|fetch/.test(normalized)) return 'Fraia could not connect. Check your connection and try again.';
  return 'Fraia could not complete this response. Try again.';
}
import {
  acceptConversationProposal,
  analyseConversationAlternative,
  applyConversationOperation,
  applyConversationOperations,
  analyseConversationSnapshot,
  applyConversationWorkingCopyOperation,
  commitConversationWorkingCopy,
  compareConversationEvidence,
  createConversationProjection,
  initializeConversation,
  openConversationWorkingCopy,
  respondConversationAgent,
  sendConversationMessage,
  rejectConversationProposal,
  type ConversationArtefactProjection,
  type ConversationAnalysisProjection,
  type ConversationComparisonProjection,
  type ConversationEvidenceProjection,
  type ConversationMessageProjection,
  type ConversationProposalProjection,
  type ConversationStructuralOperation,
  type ConversationWorkspaceProjection,
  type WorkingCopyProjection,
} from '@/lib/conversationWorkspace';
import type { AgentTarget } from '@/lib/types';

function PreviewSurface({
  artefact,
  expanded = false,
  label = 'Current structure',
  onExpand,
  onOpenEditor,
}: {
  artefact: ConversationArtefactProjection;
  expanded?: boolean;
  label?: string;
  onExpand?: () => void;
  onOpenEditor?: () => void | Promise<void>;
}) {
  return (
    <div data-testid={expanded ? 'expanded-artefact-preview' : 'artefact-preview'} className={cn('flex min-h-0 flex-col gap-2', expanded ? 'h-[min(70vh,720px)]' : 'h-52')}>
      <div role="region" aria-label={`${label} (${artefact.artefactId})`} data-testid="read-only-preview" data-preview-interaction="orbit-pan-zoom" className="relative min-h-0 flex-1 overflow-hidden rounded-lg border bg-muted/20">
        <Viewport3D
          scene={artefact.scene}
          selectionEnabled={false}
          cameraScopeKey={`artefact-${artefact.artefactId}`}
          labelVisibility={{ node: false, member: true, support: true, load: true }}
        />
        {!artefact.scene.nodes.length && !artefact.scene.members.length && !artefact.scene.plates?.length ? (
          <div data-testid="empty-preview-message" className="pointer-events-none absolute inset-x-4 bottom-4 rounded-lg border bg-background/90 px-3 py-2 text-center text-xs text-muted-foreground">
            No structure exists in this revision.
          </div>
        ) : null}
      </div>
      <div className="flex items-center justify-between gap-2">
        <span className="truncate text-xs text-muted-foreground">{label}</span>
        <div className="flex items-center gap-1">
          {onExpand ? <Button variant="ghost" size="sm" onClick={onExpand}><Maximize2 data-icon="inline-start" /> Inspect</Button> : null}
          {onOpenEditor ? <Button variant="outline" size="sm" onClick={onOpenEditor}><PencilLine data-icon="inline-start" /> Open in editor</Button> : null}
        </div>
      </div>
    </div>
  );
}

function ProposalComparison({
  proposals,
  comparison,
  onCompare,
}: {
  proposals: ConversationProposalProjection[];
  comparison: ConversationComparisonProjection;
  onCompare: () => void;
}) {
  return (
    <Card size="sm" data-testid="proposal-comparison">
      <CardHeader>
        <CardTitle>Compare candidates</CardTitle>
        <CardDescription>{comparison.summary}</CardDescription>
        <CardAction><Badge variant="outline">{Math.max(0, proposals.length - 1)} other direction{proposals.length === 2 ? '' : 's'}</Badge></CardAction>
      </CardHeader>
      <CardContent className="grid gap-2 sm:grid-cols-2">
        {proposals.map((proposal, index) => (
          <div key={proposal.proposalId} data-testid={`proposal-candidate-${proposal.proposalId}`} className="rounded-lg border p-3">
            <div className="flex items-center justify-between gap-2">
              <span className="text-sm font-medium">Candidate {index + 1}</span>
              <Badge variant="secondary">{proposal.analysed ? 'Analysed' : proposal.status}</Badge>
            </div>
            <p className="mt-1 text-sm">{proposal.title}</p>
          </div>
        ))}
      </CardContent>
      <CardFooter className="items-start justify-between gap-3">
        <div className="grid gap-1 text-xs text-muted-foreground">
          <span data-testid="comparison-evidence-boundary">{comparison.status === 'blocked' ? 'Analyse both directions to compare them.' : 'Both directions are ready to compare.'}</span>
          {comparison.details ? <span data-testid="comparison-metrics">Max utilisation {comparison.details.maxUtilizations.map((value) => value.toFixed(3)).join(' vs ')}</span> : null}
        </div>
        <Button variant="outline" size="sm" disabled={comparison.status === 'blocked'} data-testid="compare-evidence" onClick={onCompare}>Compare evidence</Button>
      </CardFooter>
    </Card>
  );
}

function ProposalCard({
  proposal,
  previewArtefact,
  index,
  busy,
  onAccept,
  onReject,
  onOpenEditor,
  onInspect,
  onAnalyseCandidate,
  onShowAlternatives,
  showAlternatives,
}: {
  proposal: ConversationProposalProjection;
  previewArtefact: ConversationArtefactProjection;
  index: number;
  busy: boolean;
  onAccept: () => void;
  onReject: () => void;
  onOpenEditor: () => void | Promise<void>;
  onInspect: () => void;
  onAnalyseCandidate?: () => void;
  onShowAlternatives?: () => void;
  showAlternatives: boolean;
}) {
  return (
    <Card size="sm" data-testid="conversation-proposal" data-proposal-id={proposal.proposalId}>
      <CardHeader>
        <CardTitle>{proposal.title}</CardTitle>
        <CardDescription>{proposal.summary}</CardDescription>
        <CardAction><Badge variant="outline">Proposal</Badge></CardAction>
      </CardHeader>
      <CardContent>
        <PreviewSurface
          artefact={previewArtefact}
          label="Proposed structure"
          onExpand={onInspect}
          onOpenEditor={onOpenEditor}
        />
      </CardContent>
      <CardFooter className="flex-wrap justify-end gap-2">
        <Button variant="outline" size="sm" disabled={busy} onClick={onOpenEditor}><PencilLine data-icon="inline-start" /> Edit this direction</Button>
        {!showAlternatives && index === 0 && onShowAlternatives ? <Button variant="ghost" size="sm" disabled={busy} onClick={onShowAlternatives}>Explore another</Button> : null}
        {showAlternatives ? <Button variant="ghost" size="sm" disabled={busy} onClick={onReject}><X data-icon="inline-start" /> Keep exploring</Button> : null}
        {index > 0 && onAnalyseCandidate ? <Button variant="outline" size="sm" disabled={busy} onClick={onAnalyseCandidate}>Analyse candidate</Button> : null}
        <Button size="sm" disabled={busy} onClick={onAccept}><Check data-icon="inline-start" /> {index === 0 ? 'Accept this direction' : 'Accept this one'}</Button>
      </CardFooter>
    </Card>
  );
}

function ProposalRecord({ proposal }: { proposal: ConversationProposalProjection }) {
  return (
    <div data-testid="proposal-record" className="flex items-center justify-between gap-3 rounded-lg border px-3 py-2 text-sm">
      <span>{proposal.title}</span>
      <Badge variant={proposal.status === 'accepted' ? 'secondary' : 'outline'}>{proposal.status === 'accepted' ? 'Accepted' : 'Not selected'}</Badge>
    </div>
  );
}

function AnalysisResultCard({ analysis }: { analysis: ConversationAnalysisProjection }) {
  const label = analysis.status === 'success' ? 'Analysis complete' : analysis.status === 'stale' ? 'Stale evidence' : analysis.status === 'unsupported' ? 'Analysis unsupported' : 'Analysis failed';
  const variant = analysis.status === 'success' ? 'secondary' : analysis.status === 'stale' ? 'outline' : 'destructive';
  return (
    <Card size="sm" data-testid="analysis-result-card">
      <CardHeader>
        <CardTitle>Result</CardTitle>
        <CardDescription>{analysis.summary}</CardDescription>
        <CardAction><Badge variant={variant}>{label}</Badge></CardAction>
      </CardHeader>
      <CardContent className="pt-0">
        <span className="text-xs text-muted-foreground">Bound to the current design</span>
      </CardContent>
    </Card>
  );
}

function MessageRow({
  message,
  onInspect,
  onOpenEditor,
  onAcceptProposal,
  onRejectProposal,
  onAnalyseCandidate,
  proposalBusy,
  comparison,
  onCompare,
  onShowAlternatives,
  showAlternatives,
  currentArtefact,
}: {
  message: ConversationMessageProjection;
  onInspect: (artefact: ConversationArtefactProjection) => void;
  onOpenEditor: (proposal?: ConversationProposalProjection) => void | Promise<void>;
  onAcceptProposal: (proposal: ConversationProposalProjection) => void;
  onRejectProposal: (proposal: ConversationProposalProjection) => void;
  onAnalyseCandidate: (proposal: ConversationProposalProjection) => void;
  proposalBusy: boolean;
  comparison: ConversationComparisonProjection;
  onCompare: () => void;
  onShowAlternatives: () => void;
  showAlternatives: boolean;
  currentArtefact: ConversationArtefactProjection;
}) {
  const allProposals = message.proposals ?? (message.proposal ? [message.proposal] : []);
  const proposals = showAlternatives ? allProposals : allProposals.slice(0, 1);
  const hasCurrentStructure = Boolean(
    message.artefact
    && (message.artefact.scene.nodes.length
      || message.artefact.scene.members.length
      || message.artefact.scene.plates?.length),
  );
  const previewForProposal = (proposal: ConversationProposalProjection): ConversationArtefactProjection => ({
    ...currentArtefact,
    artefactId: `proposal-preview-${proposal.proposalId}`,
    sourceSnapshotId: proposal.proposedRevisionId,
    scene: applyConversationOperations(
      currentArtefact.scene,
      proposal.operations?.length ? proposal.operations : [proposal.operation],
    ).scene,
  });
  const details = (
    <>
        {message.artefact && hasCurrentStructure ? (
          <Card size="sm" className="max-w-xl">
            <CardHeader>
              <CardTitle>Structural preview</CardTitle>
              <CardDescription>Inspection only. The committed snapshot stays unchanged.</CardDescription>
            </CardHeader>
            <CardContent>
              <PreviewSurface artefact={message.artefact} onExpand={() => onInspect(message.artefact!)} onOpenEditor={() => onOpenEditor()} />
            </CardContent>
          </Card>
        ) : null}
        {showAlternatives && proposals.length > 1 ? <ProposalComparison proposals={proposals} comparison={comparison} onCompare={onCompare} /> : null}
        {proposals.map((proposal, index) => proposal.status === 'pending' ? (
          <ProposalCard
            key={proposal.proposalId}
            proposal={proposal}
            previewArtefact={previewForProposal(proposal)}
            index={index}
            busy={proposalBusy}
            onAccept={() => onAcceptProposal(proposal)}
            onReject={() => onRejectProposal(proposal)}
            onAnalyseCandidate={index > 0 ? () => onAnalyseCandidate(proposal) : undefined}
            onShowAlternatives={onShowAlternatives}
            showAlternatives={showAlternatives}
            onOpenEditor={() => onOpenEditor(proposal)}
            onInspect={() => onInspect(previewForProposal(proposal))}
          />
        ) : <ProposalRecord key={proposal.proposalId} proposal={proposal} />)}
        {!showAlternatives && allProposals.length > 1 && proposals.every((proposal) => proposal.status !== 'pending') ? (
          <Button variant="ghost" size="sm" className="self-start" onClick={onShowAlternatives}>Explore another</Button>
        ) : null}
        {message.analysis ? <AnalysisResultCard analysis={message.analysis} /> : null}
    </>
  );
  return (
    <ChatTranscriptMessage
      author={message.role === 'user' ? 'user' : 'assistant'}
      messageId={message.id}
      scrollAnchor={message.role === 'user'}
      testId={`conversation-message-${message.id}`}
      details={details}
    >
      {message.content}
    </ChatTranscriptMessage>
  );
}

function WorkingCopyPanel({
  projection,
  onAddOperation,
  onAddNode,
  onAddMember,
  onAddSupport,
  onAddSection,
  onAddPlate,
  onAddLoad,
  onAddRelease,
  selectedMemberId,
  selectedMemberRole,
  selectedNodeId,
  nodePosition,
  onNodePositionChange,
  onMoveNode,
  onSelectTarget,
  onCommit,
  onCancel,
  error,
  proposalHandoff,
  pending,
}: {
  projection: WorkingCopyProjection;
  onAddOperation: () => void;
  onAddNode: (operation: Extract<ConversationStructuralOperation, { kind: 'add_node' }>) => void;
  onAddMember: (operation: Extract<ConversationStructuralOperation, { kind: 'add_member' }>) => void;
  onAddSupport: (operation: Extract<ConversationStructuralOperation, { kind: 'add_support' }>) => void;
  onAddSection: (operation: Extract<ConversationStructuralOperation, { kind: 'set_section' }>) => void;
  onAddPlate: (operation: Extract<ConversationStructuralOperation, { kind: 'add_plate' }>) => void;
  onAddLoad: (operation: Extract<ConversationStructuralOperation, { kind: 'add_load' }>) => void;
  onAddRelease: (operation: Extract<ConversationStructuralOperation, { kind: 'add_release' | 'set_release' }>) => void;
  selectedMemberId: string | null;
  selectedMemberRole: string | null;
  selectedNodeId: string | null;
  nodePosition: { x: number; y: number; z: number } | null;
  onNodePositionChange: (axis: 'x' | 'y' | 'z', value: number) => void;
  onMoveNode: () => void;
  onSelectTarget: (target: AgentTarget | null) => void;
  onCommit: () => void;
  onCancel: () => void;
  error: string | null;
  proposalHandoff: ConversationProposalProjection | null;
  pending: boolean;
}) {
  const firstNode = projection.scene.nodes[0]?.id ?? '';
  const secondNode = projection.scene.nodes[1]?.id ?? '';
  const [nodeId, setNodeId] = useState('n-new');
  const [nodePositionDraft, setNodePositionDraft] = useState({ x: 0, y: 0, z: 0 });
  const [memberId, setMemberId] = useState('m-new');
  const [memberStart, setMemberStart] = useState(firstNode);
  const [memberEnd, setMemberEnd] = useState(secondNode);
  const [memberRole, setMemberRole] = useState('beam');
  const [sectionId, setSectionId] = useState('unassigned');
  const [materialId, setMaterialId] = useState('unassigned');
  const [plateId, setPlateId] = useState('p-new');
  const [plateNodes, setPlateNodes] = useState([firstNode, secondNode].filter(Boolean).join(','));
  const [plateRole, setPlateRole] = useState('slab');
  const [plateThickness, setPlateThickness] = useState(0.15);
  const [loadId, setLoadId] = useState('l-new');
  const [loadTargetKind, setLoadTargetKind] = useState<'node' | 'member' | 'plate'>('member');
  const [loadTargetId, setLoadTargetId] = useState(secondNode);
  const [loadCaseId, setLoadCaseId] = useState('dead');
  const [loadMagnitude, setLoadMagnitude] = useState(1);
  const [loadUnit, setLoadUnit] = useState('kN/m');
  const [supportId, setSupportId] = useState('s-new');
  const [supportNode, setSupportNode] = useState(firstNode);
  const [restraints, setRestraints] = useState({ ux: true, uy: true, uz: true, rx: false, ry: false, rz: false });
  const [releaseId, setReleaseId] = useState('r-new');
  const [releaseMemberId, setReleaseMemberId] = useState('');
  const [releaseEnd, setReleaseEnd] = useState<'start' | 'end'>('start');
  const [releaseRestraints, setReleaseRestraints] = useState({ ux: false, uy: false, uz: false, rx: true, ry: true, rz: true });
  const nodeItems: Array<{ label: string; value: string | null }> = [
    { label: 'Select node', value: null },
    ...projection.scene.nodes.map((node) => ({ label: node.id, value: node.id })),
  ];
  const nodeSelect = (
    value: string,
    onValueChange: (value: string) => void,
    triggerId: string,
  ) => (
    <Select items={nodeItems} value={value || null} onValueChange={(nextValue) => onValueChange(nextValue ?? '')}>
      <SelectTrigger id={triggerId} className="w-full"><SelectValue /></SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {nodeItems.map((item) => <SelectItem key={item.value ?? 'placeholder'} value={item.value}>{item.label}</SelectItem>)}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
  return (
    <div data-testid="working-copy-panel" className="flex min-h-0 flex-1 flex-col gap-4 p-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-sm font-medium">Private edit</p>
          <p className="text-sm text-muted-foreground">Edits are private until you return this copy to the conversation.</p>
        </div>
        <Badge variant="outline">{projection.operationCount} pending {projection.operationCount === 1 ? 'edit' : 'edits'}</Badge>
      </div>
      {proposalHandoff ? (
        <Card size="sm" data-testid="proposal-editor-handoff">
          <CardHeader>
            <CardTitle>Editing from a proposal</CardTitle>
            <CardDescription>{proposalHandoff.title} remains a conversation candidate. This edit is private until you return it to the conversation.</CardDescription>
          </CardHeader>
        </Card>
      ) : null}
      {error ? <Alert variant="destructive" data-testid="working-copy-error"><AlertDescription>{error}</AlertDescription></Alert> : null}
      <div className="min-h-0 flex-1 overflow-hidden rounded-lg border bg-muted/20">
        <Viewport3D
          scene={projection.scene}
          selectionEnabled
          focusedTargets={selectedMemberId ? [{ kind: 'member', id: selectedMemberId }] : []}
          onSelectTarget={onSelectTarget}
          cameraScopeKey={`working-copy-${projection.sourceRevisionId}`}
          labelVisibility={{ node: false, member: true, support: true, load: false }}
        />
      </div>
      <Card size="sm">
        <CardHeader>
          <CardTitle>Precision editor handoff</CardTitle>
          <CardDescription>Select a member or node in the viewport, then apply a validated edit. Coordinates are metres and edits stay private until you return this copy to the conversation.</CardDescription>
        </CardHeader>
        <CardContent className="flex items-center justify-between gap-3 pt-0">
          <span data-testid="selected-editor-target" className="text-muted-foreground">
            {selectedMemberId ? `Selected member ${selectedMemberId} (${selectedMemberRole ?? 'member'})` : 'Select a member to edit'}
          </span>
        </CardContent>
        {selectedNodeId && nodePosition ? (
          <CardContent className="grid grid-cols-3 gap-2 pt-0">
            <span data-testid="working-copy-node-position" className="col-span-3 text-xs text-muted-foreground">
              {`Node ${selectedNodeId}: ${nodePosition.x}, ${nodePosition.y}, ${nodePosition.z} m`}
            </span>
            {(['x', 'y', 'z'] as const).map((axis) => (
              <label key={axis} className="grid gap-1 text-xs text-muted-foreground">
                {axis.toUpperCase()} (m)
                <Input aria-label={`Node ${axis} coordinate in metres`} type="number" step="0.1" value={nodePosition[axis]} onChange={(event) => onNodePositionChange(axis, Number(event.target.value))} />
              </label>
            ))}
          </CardContent>
        ) : null}
        <CardContent className="pt-0">
          <details className="rounded-lg border px-3" data-testid="advanced-editor">
            <summary className="cursor-pointer py-3 text-sm font-medium">Advanced object tools</summary>
            <div className="pb-3">
          <FieldSet>
            <FieldLegend variant="label">Add typed structural object</FieldLegend>
            <FieldDescription>These operations are validated in the private copy and use canonical metre coordinates. Every authored object remains private until you return this copy to the conversation.</FieldDescription>
            <div className="grid gap-4 lg:grid-cols-3">
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-node-editor">
                <p className="text-sm font-medium">Node</p>
                <Field><FieldLabel htmlFor="new-node-id">Id</FieldLabel><Input id="new-node-id" value={nodeId} onChange={(event) => setNodeId(event.target.value)} /></Field>
                <FieldGroup className="grid grid-cols-3 gap-2">
                  {(['x', 'y', 'z'] as const).map((axis) => <Field key={axis}><FieldLabel htmlFor={`new-node-${axis}`}>{axis.toUpperCase()} (m)</FieldLabel><Input id={`new-node-${axis}`} type="number" step="0.1" value={nodePositionDraft[axis]} onChange={(event) => setNodePositionDraft((current) => ({ ...current, [axis]: Number(event.target.value) }))} /></Field>)}
                </FieldGroup>
                <Button variant="outline" size="sm" disabled={pending} onClick={() => onAddNode({ kind: 'add_node', id: nodeId, ...nodePositionDraft })}>Add node</Button>
              </div>
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-member-editor">
                <p className="text-sm font-medium">Member</p>
                <Field><FieldLabel htmlFor="new-member-id">Id</FieldLabel><Input id="new-member-id" value={memberId} onChange={(event) => setMemberId(event.target.value)} /></Field>
                <Field><FieldLabel htmlFor="new-member-start">Start node</FieldLabel>{nodeSelect(memberStart, setMemberStart, 'new-member-start')}</Field>
                <Field><FieldLabel htmlFor="new-member-end">End node</FieldLabel>{nodeSelect(memberEnd, setMemberEnd, 'new-member-end')}</Field>
                <Field><FieldLabel htmlFor="new-member-role">Role</FieldLabel><Input id="new-member-role" value={memberRole} onChange={(event) => setMemberRole(event.target.value)} placeholder="beam, rafter, column" /></Field>
                <FieldGroup className="grid grid-cols-2 gap-2"><Field><FieldLabel htmlFor="new-member-section">Section</FieldLabel><Input id="new-member-section" value={sectionId} onChange={(event) => setSectionId(event.target.value)} /></Field><Field><FieldLabel htmlFor="new-member-material">Material</FieldLabel><Input id="new-member-material" value={materialId} onChange={(event) => setMaterialId(event.target.value)} /></Field></FieldGroup>
                <Button variant="outline" size="sm" disabled={pending} onClick={() => onAddMember({ kind: 'add_member', id: memberId, startNode: memberStart, endNode: memberEnd, role: memberRole, sectionId, materialId })}>Add member</Button>
                <Button variant="ghost" size="sm" disabled={pending || !(selectedMemberId ?? memberId)} onClick={() => onAddSection({ kind: 'set_section', memberId: selectedMemberId ?? memberId, sectionId })}>Set selected section</Button>
              </div>
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-support-editor">
                <p className="text-sm font-medium">Support</p>
                <Field><FieldLabel htmlFor="new-support-id">Id</FieldLabel><Input id="new-support-id" value={supportId} onChange={(event) => setSupportId(event.target.value)} /></Field>
                <Field><FieldLabel htmlFor="new-support-node">Target node</FieldLabel>{nodeSelect(supportNode, setSupportNode, 'new-support-node')}</Field>
                <FieldGroup className="grid grid-cols-3 gap-2">{(['ux', 'uy', 'uz', 'rx', 'ry', 'rz'] as const).map((axis) => <Field key={axis} orientation="horizontal"><Checkbox checked={restraints[axis]} onCheckedChange={(checked) => setRestraints((current) => ({ ...current, [axis]: Boolean(checked) }))} /><FieldLabel>{axis}</FieldLabel></Field>)}</FieldGroup>
                <Button variant="outline" size="sm" disabled={pending} onClick={() => onAddSupport({ kind: 'add_support', id: supportId, targetNode: supportNode, ...restraints })}>Add support</Button>
              </div>
            </div>
            <div className="mt-4 grid gap-4 lg:grid-cols-4">
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-plate-editor">
                <p className="text-sm font-medium">Plate</p>
                <Field><FieldLabel htmlFor="new-plate-id">Id</FieldLabel><Input id="new-plate-id" value={plateId} onChange={(event) => setPlateId(event.target.value)} /></Field>
                <Field><FieldLabel htmlFor="new-plate-boundary">Boundary nodes</FieldLabel><Input id="new-plate-boundary" value={plateNodes} onChange={(event) => setPlateNodes(event.target.value)} placeholder="n1,n2,n3" /></Field>
                <FieldGroup className="grid grid-cols-2 gap-2"><Field><FieldLabel htmlFor="new-plate-role">Role</FieldLabel><Input id="new-plate-role" value={plateRole} onChange={(event) => setPlateRole(event.target.value)} /></Field><Field><FieldLabel htmlFor="new-plate-thickness">Thickness (m)</FieldLabel><Input id="new-plate-thickness" type="number" step="0.01" value={plateThickness} onChange={(event) => setPlateThickness(Number(event.target.value))} /></Field></FieldGroup>
                <Button variant="outline" size="sm" disabled={pending} onClick={() => onAddPlate({ kind: 'add_plate', id: plateId, boundaryNodes: plateNodes.split(',').map((item) => item.trim()).filter(Boolean), role: plateRole, thicknessM: plateThickness, materialId, generatedFrom: 'precision-editor' })}>Add plate</Button>
              </div>
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-load-editor">
                <p className="text-sm font-medium">Load</p>
                <Field><FieldLabel htmlFor="new-load-id">Id</FieldLabel><Input id="new-load-id" value={loadId} onChange={(event) => setLoadId(event.target.value)} /></Field>
                <FieldGroup className="grid grid-cols-2 gap-2"><Field><FieldLabel htmlFor="new-load-target-kind">Target kind</FieldLabel><Input id="new-load-target-kind" value={loadTargetKind} onChange={(event) => setLoadTargetKind(event.target.value as typeof loadTargetKind)} /></Field><Field><FieldLabel htmlFor="new-load-target-id">Target id</FieldLabel><Input id="new-load-target-id" value={loadTargetId} onChange={(event) => setLoadTargetId(event.target.value)} /></Field></FieldGroup>
                <FieldGroup className="grid grid-cols-2 gap-2"><Field><FieldLabel htmlFor="new-load-case">Load case</FieldLabel><Input id="new-load-case" value={loadCaseId} onChange={(event) => setLoadCaseId(event.target.value)} /></Field><Field><FieldLabel htmlFor="new-load-magnitude">Magnitude</FieldLabel><Input id="new-load-magnitude" type="number" value={loadMagnitude} onChange={(event) => setLoadMagnitude(Number(event.target.value))} /></Field></FieldGroup>
                <Field><FieldLabel htmlFor="new-load-unit">Unit</FieldLabel><Input id="new-load-unit" value={loadUnit} onChange={(event) => setLoadUnit(event.target.value)} placeholder="kN/m" /></Field>
                <Button variant="outline" size="sm" disabled={pending} onClick={() => onAddLoad({ kind: 'add_load', id: loadId, targetKind: loadTargetKind, targetId: loadTargetId, loadCaseId, directionX: 0, directionY: -1, directionZ: 0, magnitude: loadMagnitude, unit: loadUnit })}>Add load</Button>
              </div>
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-release-editor">
                <p className="text-sm font-medium">Release</p>
                <Field><FieldLabel htmlFor="new-release-id">Id</FieldLabel><Input id="new-release-id" value={releaseId} onChange={(event) => setReleaseId(event.target.value)} /></Field>
                <Field><FieldLabel htmlFor="new-release-member">Member id</FieldLabel><Input id="new-release-member" value={releaseMemberId || selectedMemberId || ''} onChange={(event) => setReleaseMemberId(event.target.value)} /></Field>
                <Field><FieldLabel htmlFor="new-release-end">End</FieldLabel><Input id="new-release-end" value={releaseEnd} onChange={(event) => setReleaseEnd(event.target.value as 'start' | 'end')} /></Field>
                <FieldGroup className="grid grid-cols-3 gap-2">{(['ux', 'uy', 'uz', 'rx', 'ry', 'rz'] as const).map((axis) => <Field key={axis} orientation="horizontal"><Checkbox checked={releaseRestraints[axis]} onCheckedChange={(checked) => setReleaseRestraints((current) => ({ ...current, [axis]: Boolean(checked) }))} /><FieldLabel>{axis}</FieldLabel></Field>)}</FieldGroup>
                <Button variant="outline" size="sm" disabled={pending || !(releaseMemberId || selectedMemberId)} onClick={() => onAddRelease({ kind: 'add_release' as const, id: releaseId, memberId: releaseMemberId || selectedMemberId || '', end: releaseEnd, ...releaseRestraints })}>Add release</Button>
              </div>
            </div>
          </FieldSet>
            </div>
          </details>
        </CardContent>
        <CardFooter className="flex-wrap justify-end gap-2">
          <Button variant="outline" size="sm" disabled={pending || !selectedMemberId} onClick={onAddOperation}><PencilLine data-icon="inline-start" /> Record manual change</Button>
          <Button variant="outline" size="sm" disabled={pending || !selectedNodeId || !nodePosition} onClick={onMoveNode}><PencilLine data-icon="inline-start" /> Move selected node</Button>
          <Button variant="ghost" size="sm" onClick={onCancel}>Cancel</Button>
          <Button size="sm" disabled={!projection.operationCount || pending} onClick={onCommit}><Check data-icon="inline-start" /> Return to conversation</Button>
        </CardFooter>
      </Card>
    </div>
  );
}

export function ConversationWorkspace({ state, onState }: { state: WorkbenchState; onState?: (nextState: WorkbenchState) => void }) {
  const [projection, setProjection] = useState(() => createConversationProjection(state));
  const [liveTransport, setLiveTransport] = useState(false);
  const [inspectionArtefact, setInspectionArtefact] = useState<ConversationArtefactProjection | null>(null);
  const [workingCopy, setWorkingCopy] = useState<WorkingCopyProjection | null>(null);
  const [proposalBusy, setProposalBusy] = useState(false);
  const [composer, setComposer] = useState('');
  const [sending, setSending] = useState(false);
  const [workingCopyPending, setWorkingCopyPending] = useState(false);
  const [selectedWorkingCopyMember, setSelectedWorkingCopyMember] = useState<string | null>(null);
  const [selectedWorkingCopyNode, setSelectedWorkingCopyNode] = useState<string | null>(null);
  const [workingCopyNodePosition, setWorkingCopyNodePosition] = useState<{ x: number; y: number; z: number } | null>(null);
  const [analysisBusy, setAnalysisBusy] = useState(false);
  const [analysisAttempt, setAnalysisAttempt] = useState<AnalysisAttemptResponse | null>(null);
  const [workingCopyError, setWorkingCopyError] = useState<string | null>(null);
  const [proposalHandoff, setProposalHandoff] = useState<ConversationProposalProjection | null>(null);
  const [showAlternatives, setShowAlternatives] = useState(false);
  const [manualCommitCount, setManualCommitCount] = useState(0);
  const [transportWarning, setTransportWarning] = useState<string | null>(null);
  const [activeTurnId, setActiveTurnId] = useState<number | null>(null);
  const [failedTurnText, setFailedTurnText] = useState<string | null>(null);
  const [analysisHistoryOpen, setAnalysisHistoryOpen] = useState(false);
  const turnSequence = useRef(0);
  const activeRequestId = useRef<string | null>(null);
  const composerRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    let cancelled = false;
    initializeConversation(createConversationProjection(state)).then((result) => {
      if (cancelled) return;
      setProjection(result.projection);
      setLiveTransport(result.live);
    });
    return () => { cancelled = true; };
  }, [state]);

  async function submitMessage() {
    const message = composer.trim();
    if (!message || sending) return;
    const turnId = ++turnSequence.current;
    const requestId = `conversation-turn-${projection.head.revisionId}-${turnId}-${Date.now()}`;
    activeRequestId.current = requestId;
    setActiveTurnId(turnId);
    setComposer('');
    setProjection((current) => ({
      ...current,
      messages: [...current.messages, { id: `user-${Date.now()}`, role: 'user', content: message }],
    }));
    setSending(true);
    setFailedTurnText(null);
    try {
      if (window.fraia.conversationAgentRespond) {
        const result = await respondConversationAgent(
          projection,
          message,
          requestId,
        );
        if (turnSequence.current !== turnId) return;
        setProjection((current) => {
          const currentIds = new Set(current.messages.map((item) => item.id));
          return {
            ...result.projection,
            messages: [
              ...current.messages,
              ...result.projection.messages.filter((item) => !currentIds.has(item.id)),
            ],
          };
        });
        return;
      }
      const [next, agentResponse] = await Promise.all([
        sendConversationMessage(projection, message),
        window.fraia.agentRespondSession?.({
          projectDir: projection.projectDir,
          surface: 'pre_solve',
          text: message,
          selectedOptionIds: [],
          requestId,
        }).catch((error: unknown) => ({ error: error instanceof Error ? error.message : String(error) })),
      ]);
      if (turnSequence.current !== turnId) return;
      const nextState = agentResponse?.state as WorkbenchState | undefined;
      const agentState = nextState?.agentState ?? nextState?.agent_state;
      const agentSession = agentState?.sessions?.find((item) => item.surface === 'pre_solve');
      const latestAgentMessage = [...(agentSession?.messages ?? [])].reverse().find((item) => item.author === 'assistant' && item.text.trim());
      const agentMessage: ConversationMessageProjection | null = latestAgentMessage ? {
        id: `agent-pre-solve-${latestAgentMessage.createdAt ?? latestAgentMessage.created_at ?? Date.now()}`,
        role: 'assistant',
        content: latestAgentMessage.text,
      } : agentResponse?.error ? {
        id: `agent-error-${Date.now()}`,
        role: 'system',
        content: `Fraia could not reach the design agent. ${agentResponse.error}`,
      } : null;
      setProjection((current) => {
        const currentIds = new Set(current.messages.map((item) => item.id));
        const currentUserMessages = new Set(
          current.messages
            .filter((item) => item.role === 'user')
            .map((item) => item.content),
        );
        const incoming = [...next.messages, ...(agentMessage ? [agentMessage] : [])];
        return {
          ...next,
          messages: [
            ...current.messages,
            ...incoming.filter((item) =>
              !currentIds.has(item.id)
              && !(item.role === 'user' && currentUserMessages.has(item.content))),
          ],
        };
      });
      if (nextState) onState?.(nextState);
    } catch (error) {
      if (turnSequence.current !== turnId) return;
      const detail = error instanceof Error ? error.message : String(error);
      setFailedTurnText(message);
      setTransportWarning(detail);
    } finally {
      if (turnSequence.current === turnId) {
        setSending(false);
        setActiveTurnId(null);
        activeRequestId.current = null;
        composerRef.current?.focus();
      }
    }
  }

  function cancelResponse() {
    if (!sending) return;
    const requestId = activeRequestId.current;
    if (requestId) void window.fraia.agentCancelSession?.({ requestId });
    activeRequestId.current = null;
    turnSequence.current += 1;
    setSending(false);
    setActiveTurnId(null);
    setProjection((current) => ({
      ...current,
      messages: [...current.messages, {
        id: `response-cancelled-${Date.now()}`,
        role: 'system',
        content: 'Response cancelled. Your message remains in the conversation.',
      }],
    }));
    window.requestAnimationFrame(() => composerRef.current?.focus());
  }

  async function runAnalysis() {
    if (analysisBusy) return;
    setAnalysisBusy(true);
    setTransportWarning(null);
    setAnalysisAttempt({
      attemptId: 'starting',
      projectId: projection.revisionScopeId,
      revisionId: projection.head.revisionId,
      authoredSnapshotId: projection.head.snapshotId,
      evidenceId: 'pending',
      stage: 'preparing',
      status: 'running',
      elapsedMillis: 0,
      diagnostics: [],
    });
    try {
      const identity = crypto.randomUUID();
      let attempt = await window.fraia.startAnalysisAttempt({ projectId: projection.revisionScopeId, request: { contractVersion: 'fraia.operations.v1', requestId: `analysis-request-${identity}`, operation: 'analyse_snapshot', parameters: { revision_id: projection.head.revisionId, expected_snapshot_id: projection.head.snapshotId, evidence_id: `analysis-${identity}`, settings: { request: { Frame2DRealization: { configuration_version: 'fraia.frame2d.realization.v1' } }, check_limits: { max_utilization: 1, max_drift_ratio: 300, max_deflection_ratio: 360 } } } } });
      attempt = { ...attempt, diagnostics: attempt.diagnostics ?? [] };
      setAnalysisAttempt(attempt);
      while (attempt.status === 'running' || attempt.status === 'cancelling') { await new Promise((resolve) => setTimeout(resolve, 120)); const status = await window.fraia.analysisAttemptStatus({ projectId: projection.revisionScopeId, attemptId: attempt.attemptId }); attempt = { ...status, diagnostics: status.diagnostics ?? [] }; setAnalysisAttempt(attempt); }
      if (attempt.status === 'cancelled') return;
      const result = { live: true, analysis: { evidenceId: attempt.evidenceId, snapshotId: attempt.authoredSnapshotId, status: attempt.status === 'completed' ? 'success' as const : attempt.status === 'unsupported' ? 'unsupported' as const : 'failed' as const, summary: attempt.status === 'completed' ? 'Analysis complete. The technical record is saved in History.' : (attempt.diagnostics ?? []).join(' ') || `Analysis ${attempt.status}.` } };
      setLiveTransport((current) => current || result.live);
      setProjection((current) => {
        const evidence: ConversationEvidenceProjection[] = [...current.evidence.filter((item) => item.evidenceId !== result.analysis.evidenceId && item.status !== 'stale'), {
          evidenceId: result.analysis.evidenceId,
          authoredSnapshotId: result.analysis.snapshotId,
          status: result.analysis.status === 'success' ? 'current' : result.analysis.status,
        }];
        const currentEvidence = evidence.filter((item) => item.status === 'current');
        return {
          ...current,
          evidence,
          comparison: currentEvidence.length >= 2 ? {
            status: 'available',
            summary: 'Two accepted candidates have current snapshot-bound evidence. Compare their exact results.',
            evidenceIds: currentEvidence.slice(0, 2).map((item) => item.evidenceId),
          } : current.comparison,
          messages: [...current.messages, { id: `analysis-${result.analysis.evidenceId}-${Date.now()}`, role: 'assistant', content: result.analysis.summary, analysis: result.analysis }],
        };
      });
    } catch (caught) {
      const message = caught instanceof Error ? caught.message : String(caught);
      setTransportWarning(`Analysis attempt failed: ${message}`);
    } finally {
      setAnalysisBusy(false);
    }
  }

  async function cancelAnalysis() {
    if (!analysisAttempt || analysisAttempt.attemptId === 'starting' || analysisAttempt.status !== 'running') return;
    const cancelled = await window.fraia.cancelAnalysisAttempt({ projectId: projection.revisionScopeId, attemptId: analysisAttempt.attemptId });
    setAnalysisAttempt({ ...cancelled, diagnostics: cancelled.diagnostics ?? [] });
  }

  async function compareEvidence() {
    const result = await compareConversationEvidence(projection);
    setLiveTransport((current) => current || result.live);
    setProjection((current) => ({ ...current, comparison: result.comparison }));
  }

  function viewAnalysis() {
    const resultCards = document.querySelectorAll('[data-testid="analysis-result-card"]');
    const resultCard = resultCards.item(resultCards.length - 1);
    resultCard?.scrollIntoView({
      behavior: window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth',
      block: 'center',
    });
  }

  async function acceptProposal(nextProposal: ConversationProposalProjection) {
    setProposalBusy(true);
    try {
      const result = await acceptConversationProposal(projection, nextProposal);
      setLiveTransport((current) => current || result.live);
      setTransportWarning(result.error ? `The live proposal transport rejected this candidate. The UI is showing a local typed projection for inspection only: ${result.error}` : null);
      setProjection((current) => ({
        ...result.projection,
        messages: [...result.projection.messages, {
          id: `accepted-${nextProposal.proposalId}`,
          role: 'assistant',
          content: 'This direction is now the current design. We can analyse it or refine it.',
          artefact: result.projection.artefact,
        }],
      }));
    } finally {
      setProposalBusy(false);
    }
  }

  async function rejectProposal(nextProposal: ConversationProposalProjection) {
    setProposalBusy(true);
    try {
      const result = await rejectConversationProposal(projection, nextProposal);
      setLiveTransport((current) => current || result.live);
      setTransportWarning(result.error ? `The live proposal transport rejected this action: ${result.error}` : null);
      setProjection(result.projection);
    } finally {
      setProposalBusy(false);
    }
  }

  async function analyseAlternative(nextProposal: ConversationProposalProjection) {
    setProposalBusy(true);
    try {
      const result = await analyseConversationAlternative(projection, nextProposal);
      setLiveTransport((current) => current || result.live);
      setTransportWarning(result.error ? `The candidate could not be analysed: ${result.error}` : null);
      if (!result.error) {
        const currentEvidence = result.projection.evidence.filter((item) => item.status === 'current');
        setProjection({
          ...result.projection,
          comparison: currentEvidence.length >= 2 ? {
            status: 'available',
            summary: 'Two accepted candidates have current snapshot-bound evidence. Compare their exact results.',
            evidenceIds: currentEvidence.slice(0, 2).map((item) => item.evidenceId),
          } : result.projection.comparison,
        });
      }
    } finally {
      setProposalBusy(false);
    }
  }

  async function openEditor(nextProposal?: ConversationProposalProjection) {
    setWorkingCopyPending(true);
    setWorkingCopyError(null);
    setProposalHandoff(nextProposal ?? null);
    try {
      const result = await openConversationWorkingCopy(projection);
      let nextWorkingCopy = result.workingCopy;
      if (nextProposal) {
        const operations = nextProposal.operations?.length ? nextProposal.operations : [nextProposal.operation];
        for (const operation of operations) {
          const applied = applyConversationOperation(nextWorkingCopy.scene, operation);
          if ('error' in applied) throw new Error(applied.error);
          const persisted = await applyConversationWorkingCopyOperation(projection, nextWorkingCopy, operation);
          if (!persisted) throw new Error('Fraia could not apply the proposed structure to the private editor.');
          nextWorkingCopy = {
            ...nextWorkingCopy,
            scene: applied.scene,
            operationCount: nextWorkingCopy.operationCount + 1,
            operations: [...(nextWorkingCopy.operations ?? []), operation],
            diffSummary: [...(nextWorkingCopy.diffSummary ?? []), applied.summary],
          };
        }
      }
      setWorkingCopy(nextWorkingCopy);
      setSelectedWorkingCopyMember(nextWorkingCopy.scene.members[0]?.id ?? null);
      const firstNode = nextWorkingCopy.scene.nodes[0];
      setSelectedWorkingCopyNode(firstNode?.id ?? null);
      setWorkingCopyNodePosition(firstNode ? { x: firstNode.x, y: firstNode.y, z: firstNode.z } : null);
    } catch (error) {
      setProposalHandoff(null);
      const message = error instanceof Error ? error.message : String(error);
      setWorkingCopyError(message);
      setTransportWarning(message);
    } finally {
      setWorkingCopyPending(false);
    }
  }

  async function applyWorkingCopyOperation(operation: ConversationStructuralOperation) {
    if (!workingCopy || workingCopy.closed || workingCopyPending) return;
    const next = applyConversationOperation(workingCopy.scene, operation);
    if ('error' in next) {
      setWorkingCopyError(next.error);
      return;
    }
    setWorkingCopyPending(true);
    try {
      const transportApplied = await applyConversationWorkingCopyOperation(projection, workingCopy, operation);
      if (!transportApplied) {
        setWorkingCopyError('The live working-copy transport rejected this typed operation; the private copy was not changed.');
        return;
      }
      setWorkingCopy((current) => current ? {
        ...current,
        operationCount: current.operationCount + 1,
        operations: [...(current.operations ?? []), operation],
        diffSummary: [...(current.diffSummary ?? []), next.summary],
        scene: next.scene,
      } : current);
      setWorkingCopyError(null);
    } finally {
      setWorkingCopyPending(false);
    }
  }

  async function addWorkingCopyOperation() {
    const memberId = selectedWorkingCopyMember ?? workingCopy?.scene.members[0]?.id;
    if (!memberId || !workingCopy) return;
    const currentRole = workingCopy.scene.members.find((member) => member.id === memberId)?.role;
    await applyWorkingCopyOperation({ kind: 'set_member_role', memberId, role: currentRole === 'beam' ? 'rafter' : 'beam' });
  }

  async function moveWorkingCopyNode() {
    if (!workingCopy || workingCopy.closed || workingCopyPending || !selectedWorkingCopyNode || !workingCopyNodePosition) return;
    await applyWorkingCopyOperation({ kind: 'move_node', nodeId: selectedWorkingCopyNode, ...workingCopyNodePosition });
  }

  async function addNode(operation: Extract<ConversationStructuralOperation, { kind: 'add_node' }>) { await applyWorkingCopyOperation(operation); }
  async function addMember(operation: Extract<ConversationStructuralOperation, { kind: 'add_member' }>) { await applyWorkingCopyOperation(operation); }
  async function addSupport(operation: Extract<ConversationStructuralOperation, { kind: 'add_support' }>) { await applyWorkingCopyOperation(operation); }
  async function addSection(operation: Extract<ConversationStructuralOperation, { kind: 'set_section' }>) { await applyWorkingCopyOperation(operation); }
  async function addPlate(operation: Extract<ConversationStructuralOperation, { kind: 'add_plate' }>) { await applyWorkingCopyOperation(operation); }
  async function addLoad(operation: Extract<ConversationStructuralOperation, { kind: 'add_load' }>) { await applyWorkingCopyOperation(operation); }
  async function addRelease(operation: Extract<ConversationStructuralOperation, { kind: 'add_release' | 'set_release' }>) { await applyWorkingCopyOperation(operation); }

  async function commitWorkingCopy() {
    if (!workingCopy || workingCopy.closed || !workingCopy.operationCount) return;
    setWorkingCopyPending(true);
    const result = await commitConversationWorkingCopy(projection, workingCopy);
    if (result.error) {
      setWorkingCopyError(`Could not return this working copy: ${result.error}`);
      setWorkingCopyPending(false);
      return;
    }
    if (!result.revision) {
      setWorkingCopyError(result.error ?? 'The working-copy transport did not return a revision.');
      setWorkingCopyPending(false);
      return;
    }
    const revision = result.revision;
    const revisionId = revision.revisionId;
    const snapshotId = revision.snapshotId;
    const evidence = {
      evidenceId: `analysis-${projection.head.revisionId}`,
      authoredSnapshotId: projection.head.snapshotId,
      status: 'stale' as const,
    };
    const diffSummary = workingCopy.diffSummary?.length ? workingCopy.diffSummary.join('; ') : 'No semantic diff summary was returned.';
    setProjection((current) => ({
      ...current,
      head: revision,
      evidence: (current.evidence.length ? current.evidence : [evidence]).map((item) => ({ ...item, status: 'stale' as const })),
        messages: [...current.messages.map((message) => message.analysis
        ? { ...message, analysis: { ...message.analysis, status: 'stale' as const, summary: 'This result is stale because the model changed after it was analysed.' } }
        : message), { id: revisionId, role: 'system', content: `Your manual changes are back in the conversation. Earlier analysis is stale; rerun it when you are ready. Changed: ${diffSummary}`, evidence }],
      artefact: { ...current.artefact, sourceSnapshotId: snapshotId, scene: workingCopy.scene },
    }));
    setManualCommitCount((count) => count + 1);
    setWorkingCopy(null);
    setProposalHandoff(null);
    setSelectedWorkingCopyMember(null);
    setSelectedWorkingCopyNode(null);
    setWorkingCopyNodePosition(null);
    setWorkingCopyError(null);
    setWorkingCopyPending(false);
  }

  if (workingCopy) {
    return (
      <WorkingCopyPanel
        projection={workingCopy}
        onAddOperation={() => void addWorkingCopyOperation()}
        onAddNode={(operation) => void addNode(operation)}
        onAddMember={(operation) => void addMember(operation)}
        onAddSupport={(operation) => void addSupport(operation)}
        onAddSection={(operation) => void addSection(operation)}
        onAddPlate={(operation) => void addPlate(operation)}
        onAddLoad={(operation) => void addLoad(operation)}
        onAddRelease={(operation) => void addRelease(operation)}
        selectedMemberId={selectedWorkingCopyMember}
        selectedMemberRole={workingCopy.scene.members.find((member) => member.id === selectedWorkingCopyMember)?.role ?? null}
        selectedNodeId={selectedWorkingCopyNode}
        nodePosition={workingCopyNodePosition}
        onNodePositionChange={(axis, value) => setWorkingCopyNodePosition((current) => current ? { ...current, [axis]: value } : current)}
        onMoveNode={() => void moveWorkingCopyNode()}
        onSelectTarget={(target) => {
          setSelectedWorkingCopyMember(target?.kind === 'member' ? target.id : null);
          if (target?.kind === 'node') {
            setSelectedWorkingCopyNode(target.id);
            const node = workingCopy.scene.nodes.find((item) => item.id === target.id);
            if (node) setWorkingCopyNodePosition({ x: node.x, y: node.y, z: node.z });
          }
        }}
        onCommit={commitWorkingCopy}
        onCancel={() => { setWorkingCopy(null); setProposalHandoff(null); setWorkingCopyError(null); setSelectedWorkingCopyMember(null); setSelectedWorkingCopyNode(null); setWorkingCopyNodePosition(null); }}
        error={workingCopyError}
        proposalHandoff={proposalHandoff}
        pending={workingCopyPending}
      />
    );
  }

  const hasStructure = Boolean(
    projection.artefact.scene.nodes.length
    || projection.artefact.scene.members.length
    || projection.artefact.scene.plates?.length,
  );
  const firstUse = !hasStructure && projection.messages.length === 0;
  const projectName = state.overview?.projectName ?? state.overview?.project_name ?? 'Untitled Project';
  const designName = state.overview?.designName ?? state.overview?.design_name ?? 'Design 1';

  return (
    <div data-testid="conversation-workspace" className="flex min-h-0 flex-1 flex-col">
      {!firstUse ? <div className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3" data-purpose="manage-current-design">
        <div className="min-w-0">
          <p className="truncate text-sm font-medium">{projection.purpose}</p>
          <p className="truncate text-xs text-muted-foreground">{projectName} / {designName}</p>
        </div>
        <div className="flex max-w-full flex-wrap items-center justify-end gap-2">
          {projection.evidence.some((item) => item.status === 'stale') ? <Badge variant="outline" data-testid="stale-evidence">Stale evidence</Badge> : null}
          {projection.evidence.some((item) => item.status === 'current') ? <Badge variant="secondary" data-testid="analysis-complete">Analysis complete</Badge> : null}
          <Button variant="ghost" size="sm" onClick={() => setAnalysisHistoryOpen(true)}><History data-icon="inline-start" />History</Button>
          {projection.head.revisionId !== 'root-revision' && !projection.head.revisionId.endsWith(':root') ? (
            projection.evidence.some((item) => item.status === 'current') && !projection.evidence.some((item) => item.status === 'stale') ? (
              <Button size="sm" data-testid="view-analysis" onClick={viewAnalysis}>View result</Button>
            ) : (
              <Button variant="outline" size="sm" disabled={analysisBusy} data-testid={projection.evidence.some((item) => item.status === 'stale') ? 'rerun-analysis' : 'run-analysis'} onClick={() => void runAnalysis()}>{analysisBusy ? 'Analysing…' : projection.evidence.some((item) => item.status === 'stale') ? 'Rerun analysis' : analysisAttempt && ['failed','unsupported','cancelled'].includes(analysisAttempt.status) ? 'Retry analysis' : 'Run analysis'}</Button>
            )
          ) : null}
          <span className="sr-only" aria-live="polite">{liveTransport ? 'Fraia is ready' : 'Fraia is starting'}</span>
        </div>
      </div> : null}
      <AnalysisHistorySheet open={analysisHistoryOpen} projectDir={projection.projectRootDir} designId={projection.designId} designName={designName} currentSnapshotId={projection.head.snapshotId} ancestorSnapshotIds={projection.messages.flatMap((message) => message.evidence?.authoredSnapshotId ? [message.evidence.authoredSnapshotId] : [])} onOpenChange={setAnalysisHistoryOpen} />
      {analysisAttempt ? <div className="mx-auto flex w-full max-w-4xl flex-wrap items-center gap-2 px-3 pt-2" data-testid="analysis-attempt" data-attempt-id={analysisAttempt.attemptId} data-canonical-run-id={analysisAttempt.canonicalRunId ?? ''} data-status={analysisAttempt.status}><Marker className="min-w-0 flex-1"><MarkerIcon>{['running','cancelling'].includes(analysisAttempt.status) ? <Spinner /> : null}</MarkerIcon><MarkerContent>{analysisAttempt.attemptId === 'starting' ? 'Starting analysis…' : `${analysisAttempt.stage.split('_').join(' ')} · ${(analysisAttempt.elapsedMillis / 1000).toFixed(1)} s · ${analysisAttempt.status}`}</MarkerContent></Marker>{analysisAttempt.status === 'running' && analysisAttempt.attemptId !== 'starting' ? <Button size="sm" variant="outline" onClick={() => void cancelAnalysis()}>Cancel analysis</Button> : null}{analysisAttempt.diagnostics?.length ? <Alert className="basis-full" variant={analysisAttempt.status === 'failed' ? 'destructive' : 'default'}><AlertDescription>{analysisAttempt.diagnostics.map(analysisDiagnosticMessage).join(' ')}</AlertDescription></Alert> : null}</div> : null}
      {transportWarning ? <Alert variant="destructive" data-testid="conversation-transport-warning" className="mx-auto mt-3 w-full max-w-4xl"><AlertDescription className="flex flex-col gap-2"><span>{friendlyTransportMessage(transportWarning)}</span><Collapsible><CollapsibleTrigger render={<Button variant="ghost" size="sm" className="self-start">Details</Button>} /><CollapsibleContent><span className="break-words text-xs">{transportWarning}</span></CollapsibleContent></Collapsible></AlertDescription></Alert> : null}
      <div className="min-h-0 flex-1">
        <ChatTranscript busy={sending}>
          {firstUse ? (
            <ChatTranscriptPanel messageId="blank-conversation">
              <Empty data-testid="blank-conversation" className="min-h-full">
                <EmptyHeader>
                  <EmptyMedia variant="icon"><MessageSquareText /></EmptyMedia>
                  <Badge variant="outline" data-testid="project-design-identity">{projectName} / {designName}</Badge>
                  <EmptyTitle>What would you like to design?</EmptyTitle>
                  <EmptyDescription>Describe what you need. Fraia will ask for any dimensions, supports, loads, or constraints that are still missing.</EmptyDescription>
                </EmptyHeader>
              </Empty>
            </ChatTranscriptPanel>
          ) : projection.messages.map((message) => (
            <MessageRow
              key={message.id}
              message={message.id === 'assistant-preview' ? { ...message, artefact: projection.artefact } : message}
              onInspect={setInspectionArtefact}
              onOpenEditor={openEditor}
              onAcceptProposal={acceptProposal}
              onRejectProposal={rejectProposal}
              onAnalyseCandidate={analyseAlternative}
              proposalBusy={proposalBusy}
              comparison={projection.comparison}
              onCompare={() => void compareEvidence()}
              onShowAlternatives={() => setShowAlternatives(true)}
              showAlternatives={showAlternatives}
              currentArtefact={projection.artefact}
            />
          ))}
          {sending && activeTurnId !== null ? (
            <ChatTranscriptActivity messageId={`agent-activity-${activeTurnId}`} label="Fraia is working…">
              <ChatTranscriptCancel onClick={cancelResponse} />
            </ChatTranscriptActivity>
          ) : null}
          {!sending && failedTurnText ? (
            <div className="flex justify-end px-3">
              <Button variant="outline" size="sm" onClick={() => {
                setComposer(failedTurnText);
                setFailedTurnText(null);
                composerRef.current?.focus();
              }}>Try again</Button>
            </div>
          ) : null}
        </ChatTranscript>
      </div>
      <div className="mx-auto w-full max-w-4xl px-5 pb-5">
        <Separator className="mb-4" />
        <Field>
          <FieldLabel htmlFor="conversation-message" className="sr-only">Conversation message</FieldLabel>
          <InputGroup>
            <InputGroupTextarea
              ref={composerRef}
              id="conversation-message"
              value={composer}
              onChange={(event) => setComposer(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
                  event.preventDefault();
                  void submitMessage();
                }
              }}
              rows={2}
            />
            <InputGroupAddon align="inline-end">
              <InputGroupButton size="icon-xs" aria-label="Send message" aria-disabled={!composer.trim() || sending} onClick={() => void submitMessage()}>
                <Send data-icon="inline-start" />
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
          <FieldDescription>Cmd/Ctrl + Enter to send</FieldDescription>
        </Field>
        <p data-testid="manual-commit-count" className="sr-only">Manual revisions committed: {manualCommitCount}</p>
      </div>
      <Dialog open={Boolean(inspectionArtefact)} onOpenChange={(open) => { if (!open) setInspectionArtefact(null); }}>
        <DialogContent className="max-w-5xl" data-testid="artefact-inspection-dialog">
          <DialogHeader>
            <DialogTitle>Inspect structural preview</DialogTitle>
            <DialogDescription>Orbit, pan, and zoom the exact snapshot. Inspection does not edit the model.</DialogDescription>
          </DialogHeader>
          {inspectionArtefact ? <PreviewSurface artefact={inspectionArtefact} expanded onOpenEditor={async () => { setInspectionArtefact(null); await openEditor(); }} /> : null}
          <DialogFooter showCloseButton />
        </DialogContent>
      </Dialog>
    </div>
  );
}
