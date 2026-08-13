import { useEffect, useState } from 'react';
import { Check, Maximize2, PencilLine, Send, X } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Field, FieldDescription, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Message, MessageContent, MessageGroup, MessageHeader } from '@/components/ui/message';
import { Bubble, BubbleContent } from '@/components/ui/bubble';
import { MessageScroller, MessageScrollerContent, MessageScrollerItem, MessageScrollerProvider, MessageScrollerViewport } from '@/components/ui/message-scroller';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Textarea } from '@/components/ui/textarea';
import { Viewport3D } from '@/components/viewport/Viewport3D';
import { cn } from '@/lib/utils';
import type { WorkbenchState } from '@/lib/types';
import {
  acceptConversationProposal,
  analyseConversationAlternative,
  applyConversationOperation,
  analyseConversationSnapshot,
  applyConversationWorkingCopyOperation,
  commitConversationWorkingCopy,
  compareConversationEvidence,
  createConversationProjection,
  initializeConversation,
  openConversationWorkingCopy,
  sendConversationMessage,
  rejectConversationProposal,
  updateConversationFacts,
  type ConversationArtefactProjection,
  type ConversationAnalysisProjection,
  type ConversationComparisonProjection,
  type ConversationEvidenceProjection,
  type ConversationMessageProjection,
  type ConversationProjectFacts,
  type ConversationProposalProjection,
  type ConversationStructuralOperation,
  type ConversationWorkspaceProjection,
  type WorkingCopyProjection,
} from '@/lib/conversationWorkspace';
import type { AgentTarget } from '@/lib/types';

function PreviewSurface({
  artefact,
  expanded = false,
  onExpand,
  onOpenEditor,
}: {
  artefact: ConversationArtefactProjection;
  expanded?: boolean;
  onExpand?: () => void;
  onOpenEditor?: () => void | Promise<void>;
}) {
  return (
    <div data-testid={expanded ? 'expanded-artefact-preview' : 'artefact-preview'} className={cn('flex min-h-0 flex-col gap-2', expanded ? 'h-[min(70vh,720px)]' : 'h-52')}>
      <div role="region" aria-label="Read-only structural preview" data-testid="read-only-preview" data-preview-interaction="orbit-pan-zoom" className="relative min-h-0 flex-1 overflow-hidden rounded-lg border bg-muted/20">
        <Viewport3D
          scene={artefact.scene}
          selectionEnabled={false}
          cameraScopeKey={`artefact-${artefact.artefactId}`}
          labelVisibility={{ node: false, member: true, support: true, load: true }}
        />
        {!artefact.scene.nodes.length && !artefact.scene.members.length && !artefact.scene.plates?.length ? (
          <div data-testid="empty-preview-message" className="pointer-events-none absolute inset-x-4 bottom-4 rounded-lg border bg-background/90 px-3 py-2 text-center text-xs text-muted-foreground">
            The design is empty. Fraia will propose a first concept here.
          </div>
        ) : null}
      </div>
      <div className="flex items-center justify-between gap-2">
        <span className="truncate text-xs text-muted-foreground">Current design</span>
        <div className="flex items-center gap-1">
          {onExpand ? <Button variant="ghost" size="sm" onClick={onExpand}><Maximize2 data-icon="inline-start" /> Inspect</Button> : null}
          {onOpenEditor ? <Button variant="outline" size="sm" onClick={onOpenEditor}><PencilLine data-icon="inline-start" /> Open in editor</Button> : null}
        </div>
      </div>
    </div>
  );
}

function BriefCapture({
  facts,
  open,
  onToggle,
  onSave,
  saved = false,
}: {
  facts: ConversationProjectFacts;
  open: boolean;
  onToggle: () => void;
  onSave: (facts: ConversationProjectFacts) => void;
  saved?: boolean;
}) {
  const [draft, setDraft] = useState(facts);
  useEffect(() => setDraft(facts), [facts]);
  const setList = (key: 'constraints' | 'loadsAndAssumptions' | 'unknowns', value: string) => setDraft((current) => ({ ...current, [key]: value.split('\n').map((item) => item.trim()).filter(Boolean) }));
  return (
    <Card size="sm" data-testid="project-brief">
      <CardHeader>
        <CardTitle>Project brief</CardTitle>
        <CardDescription>Optional facts that keep the first design conversation grounded. Unknowns can stay explicit.</CardDescription>
        <CardAction className="flex items-center gap-2">{saved ? <span role="status" className="text-xs text-muted-foreground">Saved</span> : null}<Button variant="ghost" size="sm" aria-expanded={open} onClick={onToggle}>{open ? 'Hide brief' : 'Add brief'}</Button></CardAction>
      </CardHeader>
      {open ? (
        <>
          <CardContent>
            <FieldGroup className="grid gap-3 sm:grid-cols-2">
              <Field>
                <FieldLabel htmlFor="brief-building-type">Building type</FieldLabel>
                <Input id="brief-building-type" value={draft.buildingType ?? ''} onChange={(event) => setDraft((current) => ({ ...current, buildingType: event.target.value }))} placeholder="e.g. workshop" />
              </Field>
              <Field>
                <FieldLabel htmlFor="brief-objective">Objective</FieldLabel>
                <Input id="brief-objective" value={draft.objective ?? ''} onChange={(event) => setDraft((current) => ({ ...current, objective: event.target.value }))} placeholder="e.g. compare economical frames" />
              </Field>
              {(['approximateLengthM', 'approximateWidthM', 'approximateHeightM'] as const).map((key) => (
                <Field key={key}>
                  <FieldLabel htmlFor={`brief-${key}`}>{key === 'approximateLengthM' ? 'Length' : key === 'approximateWidthM' ? 'Width' : 'Height'} (m)</FieldLabel>
                  <Input id={`brief-${key}`} type="number" step="0.1" value={draft[key] ?? ''} onChange={(event) => setDraft((current) => ({ ...current, [key]: event.target.value === '' ? undefined : Number(event.target.value) }))} />
                </Field>
              ))}
              <Field className="sm:col-span-2">
                <FieldLabel htmlFor="brief-constraints">Constraints</FieldLabel>
                <Textarea id="brief-constraints" rows={2} value={draft.constraints.join('\n')} onChange={(event) => setList('constraints', event.target.value)} placeholder="One constraint per line" />
              </Field>
              <Field className="sm:col-span-2">
                <FieldLabel htmlFor="brief-loads">Loads and assumptions</FieldLabel>
                <Textarea id="brief-loads" rows={2} value={draft.loadsAndAssumptions.join('\n')} onChange={(event) => setList('loadsAndAssumptions', event.target.value)} placeholder="One load or assumption per line" />
              </Field>
              <Field className="sm:col-span-2">
                <FieldLabel htmlFor="brief-unknowns">Unknowns</FieldLabel>
                <Textarea id="brief-unknowns" rows={2} value={draft.unknowns.join('\n')} onChange={(event) => setList('unknowns', event.target.value)} placeholder="One unresolved question per line" />
                <FieldDescription>Fraia will keep these visible rather than silently deciding them.</FieldDescription>
              </Field>
            </FieldGroup>
          </CardContent>
          <CardFooter className="justify-end gap-2">
            <Button size="sm" onClick={() => onSave(draft)}>Save brief</Button>
          </CardFooter>
        </>
      ) : null}
    </Card>
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
  index,
  busy,
  onAccept,
  onReject,
  onOpenEditor,
  onAnalyseCandidate,
  onShowAlternatives,
  showAlternatives,
}: {
  proposal: ConversationProposalProjection;
  index: number;
  busy: boolean;
  onAccept: () => void;
  onReject: () => void;
  onOpenEditor: () => void | Promise<void>;
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
      <CardFooter className="justify-end gap-2">
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
        <CardTitle>Analysis result</CardTitle>
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
}) {
  const align = message.role === 'user' ? 'end' : 'start';
  const allProposals = message.proposals ?? (message.proposal ? [message.proposal] : []);
  const proposals = showAlternatives ? allProposals : allProposals.slice(0, 1);
  return (
    <Message align={align} data-testid={`conversation-message-${message.id}`}>
      <MessageContent>
        <MessageHeader className="px-0">{message.role === 'user' ? 'You' : message.role === 'system' ? 'Fraia' : 'Fraia'}</MessageHeader>
        <Bubble variant={message.role === 'user' ? 'default' : 'secondary'} align={align}>
          <BubbleContent>{message.content}</BubbleContent>
        </Bubble>
        {message.artefact ? (
          <Card size="sm" className="max-w-xl">
            <CardHeader>
              <CardTitle>Structural preview</CardTitle>
              <CardDescription>Inspection only. The committed snapshot stays unchanged.</CardDescription>
            </CardHeader>
            <CardContent>
              <PreviewSurface artefact={message.artefact} onExpand={() => onInspect(message.artefact!)} onOpenEditor={onOpenEditor} />
            </CardContent>
          </Card>
        ) : null}
        {showAlternatives && proposals.length > 1 ? <ProposalComparison proposals={proposals} comparison={comparison} onCompare={onCompare} /> : null}
        {proposals.map((proposal, index) => proposal.status === 'pending' ? (
          <ProposalCard
            key={proposal.proposalId}
            proposal={proposal}
            index={index}
            busy={proposalBusy}
            onAccept={() => onAcceptProposal(proposal)}
            onReject={() => onRejectProposal(proposal)}
            onAnalyseCandidate={index > 0 ? () => onAnalyseCandidate(proposal) : undefined}
            onShowAlternatives={onShowAlternatives}
            showAlternatives={showAlternatives}
            onOpenEditor={() => onOpenEditor(proposal)}
          />
        ) : <ProposalRecord key={proposal.proposalId} proposal={proposal} />)}
        {!showAlternatives && allProposals.length > 1 && proposals.every((proposal) => proposal.status !== 'pending') ? (
          <Button variant="ghost" size="sm" className="self-start" onClick={onShowAlternatives}>Explore another</Button>
        ) : null}
        {message.analysis ? <AnalysisResultCard analysis={message.analysis} /> : null}
      </MessageContent>
    </Message>
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
      {error ? <div role="alert" data-testid="working-copy-error" className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">{error}</div> : null}
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
                <Field><FieldLabel htmlFor="new-member-start">Start node</FieldLabel><Select value={memberStart || null} onValueChange={(value) => setMemberStart(value ?? '')}><SelectTrigger id="new-member-start" className="w-full"><SelectValue placeholder="Select node" /></SelectTrigger><SelectContent>{projection.scene.nodes.map((node) => <SelectItem key={node.id} value={node.id}>{node.id}</SelectItem>)}</SelectContent></Select></Field>
                <Field><FieldLabel htmlFor="new-member-end">End node</FieldLabel><Select value={memberEnd || null} onValueChange={(value) => setMemberEnd(value ?? '')}><SelectTrigger id="new-member-end" className="w-full"><SelectValue placeholder="Select node" /></SelectTrigger><SelectContent>{projection.scene.nodes.map((node) => <SelectItem key={node.id} value={node.id}>{node.id}</SelectItem>)}</SelectContent></Select></Field>
                <Field><FieldLabel htmlFor="new-member-role">Role</FieldLabel><Input id="new-member-role" value={memberRole} onChange={(event) => setMemberRole(event.target.value)} placeholder="beam, rafter, column" /></Field>
                <FieldGroup className="grid grid-cols-2 gap-2"><Field><FieldLabel htmlFor="new-member-section">Section</FieldLabel><Input id="new-member-section" value={sectionId} onChange={(event) => setSectionId(event.target.value)} /></Field><Field><FieldLabel htmlFor="new-member-material">Material</FieldLabel><Input id="new-member-material" value={materialId} onChange={(event) => setMaterialId(event.target.value)} /></Field></FieldGroup>
                <Button variant="outline" size="sm" disabled={pending} onClick={() => onAddMember({ kind: 'add_member', id: memberId, startNode: memberStart, endNode: memberEnd, role: memberRole, sectionId, materialId })}>Add member</Button>
                <Button variant="ghost" size="sm" disabled={pending || !(selectedMemberId ?? memberId)} onClick={() => onAddSection({ kind: 'set_section', memberId: selectedMemberId ?? memberId, sectionId })}>Set selected section</Button>
              </div>
              <div className="flex flex-col gap-3 rounded-lg border p-3" data-testid="add-support-editor">
                <p className="text-sm font-medium">Support</p>
                <Field><FieldLabel htmlFor="new-support-id">Id</FieldLabel><Input id="new-support-id" value={supportId} onChange={(event) => setSupportId(event.target.value)} /></Field>
                <Field><FieldLabel htmlFor="new-support-node">Target node</FieldLabel><Select value={supportNode || null} onValueChange={(value) => setSupportNode(value ?? '')}><SelectTrigger id="new-support-node" className="w-full"><SelectValue placeholder="Select node" /></SelectTrigger><SelectContent>{projection.scene.nodes.map((node) => <SelectItem key={node.id} value={node.id}>{node.id}</SelectItem>)}</SelectContent></Select></Field>
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

export function ConversationWorkspace({ state }: { state: WorkbenchState }) {
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
  const [workingCopyError, setWorkingCopyError] = useState<string | null>(null);
  const [proposalHandoff, setProposalHandoff] = useState<ConversationProposalProjection | null>(null);
  const [briefOpen, setBriefOpen] = useState(false);
  const [briefSaved, setBriefSaved] = useState(false);
  const [showAlternatives, setShowAlternatives] = useState(false);
  const [manualCommitCount, setManualCommitCount] = useState(0);
  const [transportWarning, setTransportWarning] = useState<string | null>(null);

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
    setComposer('');
    setProjection((current) => ({
      ...current,
      messages: [...current.messages, { id: `user-${Date.now()}`, role: 'user', content: message }],
    }));
    setSending(true);
    try {
      const next = await sendConversationMessage(projection, message);
      setProjection((current) => ({ ...next, messages: current.messages.length > next.messages.length ? current.messages : next.messages }));
    } finally {
      setSending(false);
    }
  }

  async function runAnalysis() {
    if (analysisBusy) return;
    setAnalysisBusy(true);
    try {
      const result = await analyseConversationSnapshot(projection);
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
    } finally {
      setAnalysisBusy(false);
    }
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
        messages: [...result.projection.messages, { id: `accepted-${nextProposal.proposalId}`, role: 'assistant', content: 'This direction is now the current design. We can analyse it or refine it.' }],
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
      setWorkingCopy(result.workingCopy);
      setSelectedWorkingCopyMember(result.workingCopy.scene.members[0]?.id ?? null);
      const firstNode = result.workingCopy.scene.nodes[0];
      setSelectedWorkingCopyNode(firstNode?.id ?? null);
      setWorkingCopyNodePosition(firstNode ? { x: firstNode.x, y: firstNode.y, z: firstNode.z } : null);
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

  return (
    <div data-testid="conversation-workspace" className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between gap-3 border-b px-5 py-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-medium">{projection.purpose}</p>
          <p className="truncate text-xs text-muted-foreground">Your design conversation</p>
        </div>
        <div className="flex items-center gap-2">
          {projection.evidence.some((item) => item.status === 'stale') ? <Badge variant="outline" data-testid="stale-evidence">Stale evidence</Badge> : null}
          {projection.evidence.some((item) => item.status === 'current') ? <Badge variant="secondary" data-testid="analysis-complete">Analysis complete</Badge> : null}
          {projection.head.revisionId !== 'root-revision' && !projection.head.revisionId.endsWith(':root') ? (
            projection.evidence.some((item) => item.status === 'current') && !projection.evidence.some((item) => item.status === 'stale') ? (
              <Button variant="outline" size="sm" data-testid="view-analysis" onClick={viewAnalysis}>View analysis</Button>
            ) : (
              <Button variant="outline" size="sm" disabled={analysisBusy} data-testid={projection.evidence.some((item) => item.status === 'stale') ? 'rerun-analysis' : 'run-analysis'} onClick={() => void runAnalysis()}>{analysisBusy ? 'Analysing…' : projection.evidence.some((item) => item.status === 'stale') ? 'Rerun analysis' : 'Run analysis'}</Button>
            )
          ) : null}
          <span className="sr-only" aria-live="polite">{liveTransport ? 'Conversation connected' : 'Conversation is starting'}</span>
        </div>
      </div>
      {transportWarning ? <div role="alert" data-testid="conversation-transport-warning" className="mx-auto mt-3 w-full max-w-4xl rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">{transportWarning}</div> : null}
      <div className="mx-auto w-full max-w-4xl px-5 pt-4">
        <BriefCapture
          facts={projection.projectFacts}
          open={briefOpen}
          saved={briefSaved}
          onToggle={() => { setBriefSaved(false); setBriefOpen((open) => !open); }}
          onSave={(projectFacts) => {
            setBriefSaved(false);
            void updateConversationFacts(projection, projectFacts).then((result) => {
              setLiveTransport((current) => current || result.live);
              setTransportWarning(result.error ? `The project brief was not persisted: ${result.error}` : null);
              if (!result.error) {
                setProjection(result.projection);
                setBriefSaved(true);
                setBriefOpen(false);
              }
            });
          }}
        />
      </div>
      <MessageScrollerProvider>
        <MessageScroller className="min-h-0 flex-1">
          <MessageScrollerViewport>
            <MessageScrollerContent className="mx-auto w-full max-w-4xl px-5 py-6">
              <MessageGroup>
                {projection.messages.map((message) => (
                  <MessageScrollerItem key={message.id}>
                    <MessageRow
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
                    />
                  </MessageScrollerItem>
                ))}
              </MessageGroup>
            </MessageScrollerContent>
          </MessageScrollerViewport>
        </MessageScroller>
      </MessageScrollerProvider>
      <div className="mx-auto w-full max-w-4xl px-5 pb-5">
        <Separator className="mb-4" />
        <div className="flex items-end gap-2">
          <Textarea
            value={composer}
            onChange={(event) => setComposer(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
                event.preventDefault();
                void submitMessage();
              }
            }}
            placeholder="Continue the design conversation…"
            aria-label="Conversation message"
            rows={2}
          />
          <Button size="icon" aria-label="Send message" disabled={!composer.trim() || sending} onClick={() => void submitMessage()}>
            <Send />
          </Button>
        </div>
        <p className="mt-2 text-xs text-muted-foreground">Cmd/Ctrl + Enter to send</p>
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
