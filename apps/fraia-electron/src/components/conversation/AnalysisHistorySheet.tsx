import { useEffect, useMemo, useState } from 'react';
import { AlertTriangle, History } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { DesignRunList, DesignRunStatusProjection, InspectedDesignRun } from '@/lib/engineeringEvidence';

function short(value?: string) { return value ? `${value.slice(0, 12)}…` : 'Not recorded'; }
function statusLabel(status: DesignRunStatusProjection) {
  if (status.status === 'failed') return 'Failed';
  if (status.status === 'unsupported') return 'Unsupported';
  if (status.staleness === 'current') return 'Current';
  return 'Stale';
}
function stalenessReasonLabel(code: string) {
  if (code === 'interpretation.revision_superseded') return 'A drawing interpretation was corrected after this run.';
  if (code === 'interpretation.inference_no_longer_eligible') return 'A drawing inference used by this run is no longer eligible.';
  return 'An analysis dependency changed after this run.';
}
function recordedRunLabel(run: { createdAt?: string; created_at?: string }) {
  const value = run.createdAt ?? run.created_at;
  if (!value) return 'Recorded run';
  const timestamp = new Date(value);
  return Number.isFinite(timestamp.getTime()) ? timestamp.toLocaleString() : 'Recorded run';
}

export function AnalysisHistorySheet({ open, projectDir, designId, designName, currentSnapshotId, ancestorSnapshotIds, onOpenChange }: {
  open: boolean;
  projectDir: string;
  designId: string;
  designName: string;
  currentSnapshotId: string;
  ancestorSnapshotIds: string[];
  onOpenChange: (open: boolean) => void;
}) {
  const [runs, setRuns] = useState<DesignRunList | null>(null);
  const [statuses, setStatuses] = useState<DesignRunStatusProjection[]>([]);
  const [selected, setSelected] = useState<InspectedDesignRun | null>(null);
  const [error, setError] = useState<string | null>(null);
  const statusById = useMemo(() => new Map(statuses.map((status) => [status.runId, status])), [statuses]);
  useEffect(() => {
    if (!open) return;
    setError(null);
    void Promise.all([
      window.fraia.listDesignRuns({ projectDir, designId }),
      window.fraia.listDesignRunStatuses({ projectDir, designId, inspectedSnapshotId: currentSnapshotId, ancestorSnapshotIds }),
    ]).then(([nextRuns, nextStatuses]) => { setRuns(nextRuns); setStatuses(nextStatuses); }).catch((caught) => setError(caught instanceof Error ? caught.message : String(caught)));
  }, [open, projectDir, designId, currentSnapshotId, ancestorSnapshotIds.join('|')]);
  async function inspect(runId: string) {
    setError(null);
    try { setSelected(await window.fraia.inspectDesignRun({ projectDir, designId, runId })); }
    catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); }
  }
  return <Sheet open={open} onOpenChange={onOpenChange}>
    <SheetContent className="sm:max-w-2xl" data-testid="analysis-history-sheet">
      <SheetHeader><SheetTitle>Analysis history</SheetTitle><SheetDescription>Results for {designName}. Open a run when you need its technical record.</SheetDescription></SheetHeader>
      {error ? <Alert variant="destructive" className="mx-4"><AlertTriangle /><AlertTitle>Analysis history failed</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
      <Tabs defaultValue="runs" className="min-h-0 flex-1 px-4 pb-4">
        <TabsList className="w-full"><TabsTrigger value="runs">Runs</TabsTrigger><TabsTrigger value="details" disabled={!selected}>Run details</TabsTrigger></TabsList>
        <TabsContent value="runs" className="min-h-0"><ScrollArea className="h-[calc(100vh-13rem)] pr-3">
          {!runs?.runs.length ? <Empty><EmptyHeader><EmptyMedia variant="icon"><History /></EmptyMedia><EmptyTitle>No canonical analysis runs</EmptyTitle><EmptyDescription>Run analysis from the conversation after Fraia has an accepted structure.</EmptyDescription></EmptyHeader></Empty> : <div className="flex flex-col gap-3 py-3">{runs.runs.map((run) => {
            const status = statusById.get(run.runId);
            return <Item key={run.runId} variant="outline" data-testid="design-run-row"><ItemContent><ItemTitle>{run.runKind.split('_').join(' ')}</ItemTitle><ItemDescription>{recordedRunLabel(run)}</ItemDescription>{status?.stalenessReasons?.map((reason) => <ItemDescription key={`${reason.code}-${reason.interpretationRevisionId ?? reason.inferenceId ?? ''}`} data-staleness-code={reason.code}>{stalenessReasonLabel(reason.code)}</ItemDescription>)}</ItemContent><Badge variant={status?.status === 'failed' ? 'destructive' : status?.staleness === 'current' ? 'secondary' : 'outline'}>{status ? statusLabel(status) : run.status}</Badge><ItemActions><Button size="sm" variant="outline" onClick={() => void inspect(run.runId)}>Open</Button></ItemActions></Item>;
          })}{runs.legacyRuns.map((run) => <Alert key={run.directoryName}><AlertTriangle /><AlertTitle>Legacy run</AlertTitle><AlertDescription>{run.directoryName} has no canonical manifest and cannot be used as current evidence.</AlertDescription></Alert>)}</div>}
        </ScrollArea></TabsContent>
        <TabsContent value="details" className="min-h-0"><ScrollArea className="h-[calc(100vh-13rem)] pr-3">{selected?.format === 'canonical' ? <div className="flex flex-col gap-3 py-3" data-testid="canonical-run-details">
          <Card size="sm"><CardHeader><CardTitle>{selected.manifest.runKind.split('_').join(' ')}</CardTitle><CardDescription className="break-all">Run {selected.manifest.runId}</CardDescription><CardAction><Badge variant={selected.manifest.status === 'completed' ? 'secondary' : selected.manifest.status === 'failed' ? 'destructive' : 'outline'}>{selected.manifest.status}</Badge></CardAction></CardHeader><CardContent className="flex flex-col gap-2 break-all"><span>Authored revision: {selected.manifest.authoredRevisionId}</span><span>Authored snapshot: {selected.manifest.authoredSnapshotId}</span><span>Resolved snapshot: {selected.manifest.resolvedSnapshotId ?? 'Not produced'}</span><Separator /><span>Solver: {selected.manifest.solverIdentity}</span><span>Runtime: {selected.manifest.runtimeIdentity}</span><span>Settings identity: {selected.manifest.settingsIdentity}</span><span>Request identity: {selected.manifest.requestIdentity}</span><span>Input identity: {selected.manifest.inputIdentity ?? 'Not produced'}</span><span>Result identity: {selected.manifest.resultIdentity ?? 'Not produced'}</span></CardContent></Card>
          {selected.manifest.diagnostics.length ? selected.manifest.diagnostics.map((diagnostic) => <Alert key={`${diagnostic.code}-${diagnostic.message}`} variant={diagnostic.severity === 'error' ? 'destructive' : 'default'}><AlertTitle>{diagnostic.code}</AlertTitle><AlertDescription>{diagnostic.message}</AlertDescription></Alert>) : <Alert><AlertTitle>No run diagnostics</AlertTitle><AlertDescription>The canonical run did not record warnings or errors.</AlertDescription></Alert>}
          <Card size="sm"><CardHeader><CardTitle>Attachments and provenance</CardTitle><CardDescription>Checksums identify the immutable run artefacts.</CardDescription></CardHeader><CardContent className="flex flex-col gap-2">{selected.manifest.attachments.map((attachment) => <Item key={`${attachment.role}-${attachment.sha256}`} variant="outline"><ItemContent><ItemTitle>{attachment.name}</ItemTitle><ItemDescription>{attachment.role.split('_').join(' ')} · {attachment.mediaType} · {attachment.byteSize} bytes · SHA-256 {attachment.sha256}</ItemDescription></ItemContent></Item>)}</CardContent></Card>
        </div> : selected?.format === 'legacy' ? <Alert className="mt-3"><AlertTriangle /><AlertTitle>Legacy analysis folder</AlertTitle><AlertDescription>{selected.directoryName} is shown for history only. It has no canonical snapshot-bound manifest.</AlertDescription></Alert> : null}</ScrollArea></TabsContent>
      </Tabs>
    </SheetContent>
  </Sheet>;
}
