import { useEffect, useMemo, useState } from 'react';
import { AlertTriangle, Check, GitCompare, X } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty';
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { DrawingConflict, DrawingInterpretation, DrawingObservation } from '@/lib/engineeringEvidence';

type ShelfItem = {
  id: string;
  label: string;
  kind: string;
  source?: { source_id: string; source_sha256: string };
  page_number?: number;
  crop?: { x: number; y: number; width: number; height: number; coordinate_space: string };
  drawing_context?: { view_role?: DrawingObservation['viewRole'] };
};

const VIEW_ROLES = ['plan', 'elevation', 'section', 'detail', 'schedule', 'reference'] as const;
function now() { return new Date().toISOString(); }
function statusOf(observation: DrawingObservation) { return observation.confirmation.status; }
function featureLabel(observation: DrawingObservation) {
  return observation.featureKind.split('_').join(' ');
}
export function DrawingInterpretationDialog({ open, projectDir, projectId, designId, designName, reference, onOpenChange }: {
  open: boolean;
  projectDir: string;
  projectId: string;
  designId: string;
  designName: string;
  reference: ShelfItem | null;
  onOpenChange: (open: boolean) => void;
}) {
  const [head, setHead] = useState<DrawingInterpretation | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rejectReason, setRejectReason] = useState('Does not match the current design intent.');
  const [rejectObservationId, setRejectObservationId] = useState<string | null>(null);
  const [correctedObservationIds, setCorrectedObservationIds] = useState<Set<string>>(new Set());
  const [origin, setOrigin] = useState({ x: '0', y: '0', z: '0' });
  const observations = useMemo(() => Object.values(head?.observations ?? {}).filter((item) => !reference || item.shelfItemId === reference.id), [head, reference]);
  const conflicts = useMemo(() => Object.values(head?.conflicts ?? {}) as DrawingConflict[], [head]);

  async function refresh() {
    setError(null);
    const list = await window.fraia.listDrawingInterpretations({ projectDir, designId });
    const nextHead = list.headRevisionId ? await window.fraia.inspectDrawingInterpretation({ projectDir, designId, revisionId: list.headRevisionId }) : null;
    setHead(nextHead);
    if (nextHead?.method === 'manual' && nextHead.parentRevisionId) {
      const parent = await window.fraia.inspectDrawingInterpretation({ projectDir, designId, revisionId: nextHead.parentRevisionId });
      setCorrectedObservationIds(new Set(Object.values(nextHead.observations).filter((observation) => {
        const prior = parent.observations[observation.id];
        return prior && (prior.viewRole !== observation.viewRole || JSON.stringify(prior.feature) !== JSON.stringify(observation.feature));
      }).map((observation) => observation.id)));
    } else setCorrectedObservationIds(new Set());
  }
  useEffect(() => { if (open) void refresh().catch((caught) => setError(caught instanceof Error ? caught.message : String(caught))); }, [open, projectDir, designId]);

  async function createReview() {
    if (!reference?.source || reference.kind !== 'pdf_crop' || !reference.crop || !reference.page_number) return;
    setBusy(true); setError(null);
    try {
      const createdAt = now();
      const observationId = `observation-${reference.id}`;
      const crop = reference.crop;
      const observation = {
        id: observationId,
        shelfItemId: reference.id,
        sourceId: reference.source.source_id,
        sourceSha256: reference.source.source_sha256,
        sourceLocator: { locatorKind: 'pdf_page', page_number: reference.page_number, coordinate_space: crop.coordinate_space },
        viewRole: reference.drawing_context?.view_role ?? 'reference',
        sourceGeometry: { sourceGeometryKind: 'region', boundary: [[crop.x, crop.y], [crop.x + crop.width, crop.y], [crop.x + crop.width, crop.y + crop.height], [crop.x, crop.y + crop.height]] },
        extraction: { method: 'manual', producer: 'fraia-desktop', producerVersion: '1', confidence: 1, uncertainty: [] },
        confirmation: { status: 'unconfirmed' },
        featureKind: 'region',
        region_role: 'selected_design_reference',
      };
      setHead(await window.fraia.createDrawingInterpretation({ projectDir, designId, expectedParentRevisionId: head?.revisionId, authority: 'user', revision: { projectId, designId, parentRevisionId: head?.revisionId, createdAt, method: 'manual', observations: { ...(head?.observations ?? {}), [observationId]: observation }, correspondences: head?.correspondences ?? {}, alignmentTransforms: head?.alignmentTransforms ?? {}, conflicts: head?.conflicts ?? {} } }));
    } catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); } finally { setBusy(false); }
  }
  async function confirm(observation: DrawingObservation) {
    if (!head) return;
    setBusy(true); setError(null);
    try { const createdAt = now(); setHead(await window.fraia.confirmDrawingObservations({ projectDir, designId, operation: { expectedParentRevisionId: head.revisionId, observationIds: [observation.id], confirmedBy: 'user', confirmedAt: createdAt, createdAt } })); }
    catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); } finally { setBusy(false); }
  }
  async function authorObservation(observation: DrawingObservation, update: Partial<DrawingObservation>) {
    if (!head) return;
    setBusy(true); setError(null);
    try { const createdAt = now(); setHead(await window.fraia.createDrawingInterpretation({ projectDir, designId, expectedParentRevisionId: head.revisionId, authority: 'user', revision: { projectId, designId, parentRevisionId: head.revisionId, createdAt, method: 'manual', observations: { ...head.observations, [observation.id]: { ...observation, ...update } }, correspondences: head.correspondences, alignmentTransforms: head.alignmentTransforms, conflicts: head.conflicts } })); }
    catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); } finally { setBusy(false); }
  }

  async function correctViewRole(observation: DrawingObservation, correctedViewRole: DrawingObservation['viewRole']) {
    if (!head) return;
    setBusy(true); setError(null);
    try {
      const correctedAt = now();
      setHead(await window.fraia.correctDrawingObservation({ projectDir, designId, operation: { expectedParentRevisionId: head.revisionId, observationId: observation.id, correctedViewRole, correctedBy: 'user', correctedAt, createdAt: correctedAt } }));
      setCorrectedObservationIds(new Set([observation.id]));
    } catch (caught: any) { setError(caught?.message || 'Could not save the drawing correction.'); } finally { setBusy(false); }
  }
  async function align(observation: DrawingObservation) {
    if (!head) return;
    const peer = Object.values(head.observations).find((candidate) => candidate.id !== observation.id && candidate.confirmation.status === 'confirmed');
    if (!peer) { setError('Confirm an observation from another drawing view before you align and reconcile these views.'); return; }
    const x = Number(origin.x); const y = Number(origin.y); const z = Number(origin.z);
    if (![x, y, z].every(Number.isFinite)) return;
    const transformId = `alignment-${observation.id}-${Date.now()}`; const correspondenceId = `correspondence-${observation.id}-${peer.id}-${Date.now()}`; const createdAt = now();
    const offset = ([sx, sy]: [number, number]) => [sx + x, sy + y, z];
    const sourceGeometry = observation.sourceGeometry;
    const designGeometry = sourceGeometry.sourceGeometryKind === 'region'
      ? { designGeometryKind: 'region', boundary: (sourceGeometry.boundary as Array<[number, number]>).map(offset), alignment_transform_id: transformId }
      : sourceGeometry.sourceGeometryKind === 'polyline'
        ? { designGeometryKind: 'polyline', coordinates: (sourceGeometry.coordinates as Array<[number, number]>).map(offset), closed: Boolean(sourceGeometry.closed), alignment_transform_id: transformId }
        : sourceGeometry.sourceGeometryKind === 'point'
          ? { designGeometryKind: 'point', coordinate: offset(sourceGeometry.coordinate as [number, number]), alignment_transform_id: transformId }
          : null;
    if (!designGeometry) { setError('This drawing observation cannot be aligned as design geometry.'); return; }
    setBusy(true); setError(null);
    try { setHead(await window.fraia.reconcileDrawingInterpretation({ projectDir, designId, operation: { expectedParentRevisionId: head.revisionId, designGeometries: { [observation.id]: designGeometry }, correspondences: { [correspondenceId]: { id: correspondenceId, observationIds: [observation.id, peer.id], relation: 'same_axis', confidence: 1, confirmation: { status: 'confirmed', confirmed_by: 'user', confirmed_at: createdAt }, uncertainty: [] } }, alignmentTransforms: { [transformId]: { id: transformId, fromShelfItemId: observation.shelfItemId, toDesignCoordinateSpace: 'fraia_design_m', matrix: [1, 0, 0, x, 0, 1, 0, y, 0, 0, 1, z, 0, 0, 0, 1], establishedByCorrespondenceIds: [correspondenceId], confirmedBy: 'user', confirmedAt: createdAt } }, conflicts: {}, createdAt } })); }
    catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); } finally { setBusy(false); }
  }
  async function resolveConflict(conflict: DrawingConflict) {
    if (!head) return; setBusy(true); setError(null);
    try { const createdAt = now(); setHead(await window.fraia.resolveDrawingInterpretationConflict({ projectDir, designId, operation: { expectedParentRevisionId: head.revisionId, conflictId: conflict.id, resolution: 'Reviewed and resolved by the user.', resolvedBy: 'user', resolvedAt: createdAt, createdAt } })); }
    catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); } finally { setBusy(false); }
  }

  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent className="max-h-[92vh] w-[min(94vw,52rem)]! max-w-[min(94vw,52rem)]!" data-testid="drawing-interpretation-dialog">
      <DialogHeader><DialogTitle>Review Fraia's interpretation</DialogTitle><DialogDescription>Check what Fraia understood from {reference?.label ?? 'this design reference'}. Correct anything that is wrong, then confirm what is safe to use.</DialogDescription></DialogHeader>
      {error ? <Alert variant="destructive"><AlertTriangle /><AlertTitle>Drawing review failed</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
      <ScrollArea className="max-h-[65vh] pr-3">
        <div className="flex flex-col gap-4">
          {!observations.length ? <Empty><EmptyHeader><EmptyTitle>Ready to review</EmptyTitle><EmptyDescription>Fraia will start from this design reference and keep its interpretation unconfirmed until you check it.</EmptyDescription></EmptyHeader><Button disabled={busy || reference?.kind !== 'pdf_crop'} onClick={() => void createReview()}>Review this reference</Button></Empty> : observations.map((observation) => <Card key={observation.id} size="sm" data-testid="drawing-observation">
            <CardHeader><CardTitle>{featureLabel(observation)}</CardTitle><CardDescription>{observation.viewRole[0].toUpperCase() + observation.viewRole.slice(1)}</CardDescription><CardAction className="flex flex-col items-end gap-1">{correctedObservationIds.has(observation.id) ? <Badge variant="secondary">You corrected</Badge> : observation.extraction.method !== 'manual' ? <Badge variant="secondary">Fraia inferred</Badge> : null}<Badge variant={statusOf(observation) === 'confirmed' ? 'secondary' : statusOf(observation) === 'rejected' ? 'destructive' : 'outline'}>{statusOf(observation)}</Badge></CardAction></CardHeader>
            <CardContent className="flex flex-col gap-3">
              <Field><FieldLabel>Drawing view</FieldLabel><Select items={VIEW_ROLES.map((value) => ({ value, label: value[0].toUpperCase() + value.slice(1) }))} value={observation.viewRole} onValueChange={(value) => { if (value && value !== observation.viewRole) void correctViewRole(observation, value as DrawingObservation['viewRole']); }}><SelectTrigger><SelectValue /></SelectTrigger><SelectContent><SelectGroup>{VIEW_ROLES.map((value) => <SelectItem key={value} value={value}>{value[0].toUpperCase() + value.slice(1)}</SelectItem>)}</SelectGroup></SelectContent></Select><FieldDescription>Assign plan, elevation, section, detail, schedule, or reference before reconciliation.</FieldDescription></Field>
              {observation.extraction.uncertainty?.map((uncertainty) => <Alert key={`${uncertainty.kind}-${uncertainty.message}`}><AlertTriangle /><AlertTitle>{uncertainty.kind.split('_').join(' ')}</AlertTitle><AlertDescription>{uncertainty.message}</AlertDescription></Alert>)}
              <Collapsible><CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>Evidence details</CollapsibleTrigger><CollapsibleContent className="pt-2"><div className="text-sm text-muted-foreground"><p>{observation.extraction.method.split('_').join(' ')} evidence · {Math.round(observation.extraction.confidence * 100)}% confidence</p><p className="break-all">Source {observation.sourceId} · {observation.sourceSha256}</p><p className="break-all">Observation {observation.id}</p></div></CollapsibleContent></Collapsible>
              {statusOf(observation) === 'confirmed' ? <Collapsible><CollapsibleTrigger render={<Button variant="outline" size="sm" />}>Alignment details</CollapsibleTrigger><CollapsibleContent className="pt-3"><FieldGroup><Field><FieldLabel htmlFor={`origin-x-${observation.id}`}>Origin X (m)</FieldLabel><Input id={`origin-x-${observation.id}`} inputMode="decimal" value={origin.x} onChange={(event) => setOrigin((current) => ({ ...current, x: event.target.value }))} /></Field><Field><FieldLabel htmlFor={`origin-y-${observation.id}`}>Origin Y (m)</FieldLabel><Input id={`origin-y-${observation.id}`} inputMode="decimal" value={origin.y} onChange={(event) => setOrigin((current) => ({ ...current, y: event.target.value }))} /></Field><Field><FieldLabel htmlFor={`level-z-${observation.id}`}>Level Z (m)</FieldLabel><Input id={`level-z-${observation.id}`} inputMode="decimal" value={origin.z} onChange={(event) => setOrigin((current) => ({ ...current, z: event.target.value }))} /><FieldDescription>Fraia does not guess this alignment.</FieldDescription></Field></FieldGroup></CollapsibleContent></Collapsible> : null}
              {statusOf(observation) === 'unconfirmed' && rejectObservationId === observation.id ? <Field><FieldLabel htmlFor={`drawing-reject-reason-${observation.id}`}>Rejection reason</FieldLabel><Input id={`drawing-reject-reason-${observation.id}`} value={rejectReason} autoFocus onChange={(event) => setRejectReason(event.target.value)} /></Field> : null}
            </CardContent>
            <CardFooter className="flex-wrap justify-end gap-2">{statusOf(observation) === 'unconfirmed' ? rejectObservationId === observation.id ? <><Button variant="outline" onClick={() => setRejectObservationId(null)}>Cancel</Button><Button variant="destructive" disabled={busy || !rejectReason.trim()} onClick={() => void authorObservation(observation, { confirmation: { status: 'rejected', rejectedBy: 'user', rejectedAt: now(), reason: rejectReason.trim() } })}><X data-icon="inline-start" />Mark not correct</Button></> : <Button variant="outline" disabled={busy} onClick={() => setRejectObservationId(observation.id)}><X data-icon="inline-start" />Not correct</Button> : null}{statusOf(observation) === 'confirmed' ? <Button variant="outline" disabled={busy || !Object.values(head?.observations ?? {}).some((candidate) => candidate.id !== observation.id && candidate.confirmation.status === 'confirmed')} onClick={() => void align(observation)}><GitCompare data-icon="inline-start" />Align views</Button> : null}{statusOf(observation) === 'unconfirmed' ? <Button disabled={busy} onClick={() => void confirm(observation)}><Check data-icon="inline-start" />Confirm</Button> : null}</CardFooter>
          </Card>)}
          {head && Object.values(head.observations).filter((observation) => observation.confirmation.status === 'confirmed').length < 2 ? <Alert><GitCompare /><AlertTitle>Add another confirmed view to reconcile</AlertTitle><AlertDescription>Confirm a plan, elevation, section, or detail from another design reference. Fraia will not invent a cross-view alignment from one drawing.</AlertDescription></Alert> : null}
          {conflicts.filter((conflict) => conflict.resolution.status === 'unresolved').map((conflict) => <Alert key={conflict.id} variant="destructive"><AlertTriangle /><AlertTitle>{conflict.conflictKind.split('_').join(' ')}</AlertTitle><AlertDescription className="flex flex-col gap-2"><span>{conflict.message}</span><Button variant="outline" size="sm" disabled={busy} onClick={() => void resolveConflict(conflict)}>Mark conflict resolved</Button></AlertDescription></Alert>)}
        </div>
      </ScrollArea>
      <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)}>Done</Button></DialogFooter>
    </DialogContent>
  </Dialog>;
}
