import { ChevronRight, History } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Empty, EmptyDescription } from '@/components/ui/empty';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { ScrollArea } from '@/components/ui/scroll-area';
import type { DesignOptionBatchState, DesignOptionRevisionState, EngineeringScheme } from '../../lib/types';

export type ActiveView =
  | { kind: 'base' }
  | { kind: 'scheme'; id: string }
  | { kind: 'development'; pathId: string; optionId: string }
  | { kind: 'evidence'; optionId: string; runId?: string | null }
  | { kind: 'analysis' }
  | { kind: 'results' };
export type WorkspacePanel = 'base-chat' | 'design-options' | 'development' | null;

function conciseSchemeTitle(scheme: EngineeringScheme) {
  return scheme.name.trim().replace(/^schema\s+/i, '').replace(/^scheme\s+/i, '').replace(/^option\s+/i, '').trim() || scheme.name;
}

function analysisLabel(revision: DesignOptionRevisionState) {
  const status = revision.analysisStatus ?? revision.analysis_status;
  if (status === 'current') return 'Analysed';
  if (status === 'failed') return 'Needs attention';
  if (status === 'stale') return 'Outdated';
  return 'Not analysed';
}

function DesignOptionItem({
  active,
  readOnly,
  optionNumber,
  scheme,
  revision,
  onInspect,
  onIncludedChange,
}: {
  active: boolean;
  readOnly: boolean;
  optionNumber: number;
  scheme: EngineeringScheme;
  revision: DesignOptionRevisionState;
  onInspect: () => void;
  onIncludedChange: (included: boolean) => void;
}) {
  const id = `include-option-${scheme.id.replace(/[^a-z0-9]+/gi, '-')}`;
  return (
    <Card aria-current={active ? 'true' : undefined}>
      <CardContent className="flex items-start gap-2 p-3">
        <Checkbox id={id} checked={revision.included} disabled={readOnly} aria-label={`Include ${scheme.name} for analysis`} onCheckedChange={(checked) => onIncludedChange(checked === true)} />
        <Button type="button" onClick={onInspect} variant="ghost" className="h-auto min-w-0 flex-1 flex-col items-stretch justify-start gap-0 p-0">
          <div className="flex items-center gap-2">
            <Badge variant={active ? 'default' : 'secondary'}>{optionNumber}</Badge>
            <div className="truncate font-medium" title={scheme.name}>{conciseSchemeTitle(scheme)}</div>
          </div>
          <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{scheme.summary}</p>
          <div className="mt-2 flex flex-wrap gap-1">
            <Badge variant="outline">{analysisLabel(revision)}</Badge>
            {scheme.analysisSummary?.maxUtilization != null ? <Badge variant="outline">u {scheme.analysisSummary.maxUtilization.toFixed(2)}</Badge> : null}
            {scheme.approximateMassKg != null ? <Badge variant="outline">{scheme.approximateMassKg.toFixed(0)} kg</Badge> : null}
          </div>
        </Button>
        <Button aria-label={`Inspect ${scheme.name}`} onClick={onInspect} size="icon-sm" variant="ghost"><ChevronRight /></Button>
      </CardContent>
    </Card>
  );
}

export function DesignOptionsPanel({
  active,
  schemes,
  batch,
  batches,
  stage,
  busy,
  onSelectScheme,
  onIncludedChange,
}: {
  active: ActiveView;
  schemes: EngineeringScheme[];
  batch: DesignOptionBatchState | null;
  batches: DesignOptionBatchState[];
  stage: 'options' | 'analysis';
  busy: boolean;
  onSelectScheme: (id: string) => void;
  onIncludedChange: (id: string, included: boolean) => void;
}) {
  const revisions = batch?.optionRevisions ?? batch?.option_revisions ?? [];
  const entries = schemes.flatMap((scheme, index) => {
    const revision = revisions.find((candidate) => (candidate.optionId ?? candidate.option_id) === scheme.id);
    return revision ? [{ scheme, revision, index }] : [];
  });
  const included = entries.filter(({ revision }) => revision.included);
  const excluded = entries.filter(({ revision }) => !revision.included);
  const historicalBatches = batches.filter((candidate) => candidate.id !== batch?.id);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="shrink-0 border-b p-3">
        <div className="flex items-center justify-between gap-2">
          <div className="flex min-w-0 items-center gap-2"><span className="truncate font-medium">{stage === 'analysis' ? 'Options in comparison' : 'Included for analysis'}</span><Badge variant="secondary">{included.length}</Badge></div>
          {stage === 'options' && historicalBatches.length ? (
            <Popover>
              <PopoverTrigger render={<Button variant="outline" size="sm" />}>
                <History data-icon="inline-start" /> History
              </PopoverTrigger>
              <PopoverContent align="start" className="w-80 p-2">
                <div className="px-2 py-1 text-sm font-medium">Previous option batches</div>
                <p className="px-2 pb-2 text-xs text-muted-foreground">Read-only references. Regenerate from Base Model to create a current batch.</p>
                <ScrollArea className="max-h-72">
                  <div className="flex flex-col gap-2">
                    {[...historicalBatches].reverse().map((historical) => {
                      const revisions = historical.optionRevisions ?? historical.option_revisions ?? [];
                      return (
                        <Card key={historical.id}>
                          <CardContent className="flex flex-col gap-1 p-3">
                            <div className="flex items-center justify-between gap-2"><span className="truncate font-medium">{historical.id}</span><Badge variant="outline">{historical.status}</Badge></div>
                            <span className="text-xs text-muted-foreground">{revisions.length} option revision{revisions.length === 1 ? '' : 's'} · {historical.generatedAt ?? historical.generated_at}</span>
                          </CardContent>
                        </Card>
                      );
                    })}
                  </div>
                </ScrollArea>
              </PopoverContent>
            </Popover>
          ) : null}
        </div>
        <p className="mt-1 text-xs text-muted-foreground">{stage === 'analysis' ? 'Return to Design Options to change the shortlist.' : 'Untick concepts to remove them from further analysis. Nothing is deleted.'}</p>
        {batch?.status === 'outdated' ? <Badge variant="destructive" className="mt-2">Base Model changed · regenerate options</Badge> : null}
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <div className="flex flex-col gap-2 p-2">
          {included.map(({ scheme, revision, index }) => (
            <DesignOptionItem key={scheme.id} active={(active.kind === 'scheme' || active.kind === 'development') && (active.kind === 'scheme' ? active.id : active.optionId) === scheme.id} readOnly={stage === 'analysis'} optionNumber={index + 1} scheme={scheme} revision={revision} onInspect={() => onSelectScheme(scheme.id)} onIncludedChange={(checked) => !busy && onIncludedChange(scheme.id, checked)} />
          ))}
          {!included.length ? (
            <Empty className="min-h-24">
              <EmptyDescription>Include at least one option before moving to Analysis &amp; Comparison.</EmptyDescription>
            </Empty>
          ) : null}
          {excluded.length ? (
            <Card>
              <CardContent className="p-2">
                <Collapsible>
                  <CollapsibleTrigger render={<Button variant="ghost" className="w-full justify-start" />}>
                    Excluded options ({excluded.length})
                  </CollapsibleTrigger>
                  <CollapsibleContent className="mt-2 flex flex-col gap-2">
                    {excluded.map(({ scheme, revision, index }) => (
                      <DesignOptionItem key={scheme.id} active={(active.kind === 'scheme' || active.kind === 'development') && (active.kind === 'scheme' ? active.id : active.optionId) === scheme.id} readOnly={stage === 'analysis'} optionNumber={index + 1} scheme={scheme} revision={revision} onInspect={() => onSelectScheme(scheme.id)} onIncludedChange={(checked) => !busy && onIncludedChange(scheme.id, checked)} />
                    ))}
                  </CollapsibleContent>
                </Collapsible>
              </CardContent>
            </Card>
          ) : null}
        </div>
      </ScrollArea>
    </div>
  );
}
