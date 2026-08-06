import { ChevronRight, History, Scale } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Empty, EmptyDescription } from '@/components/ui/empty';
import { Field, FieldLabel } from '@/components/ui/field';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
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
  const title = scheme.name.trim().replace(/^schema\s+/i, '').replace(/^scheme\s+/i, '').replace(/^option\s+/i, '').trim();
  return title.length > 1 ? title : scheme.name;
}

function conciseSchemeDetail(scheme: EngineeringScheme) {
  const band = (scheme.intent?.explorationBand ?? scheme.intent?.exploration_band)?.trim();
  const objectiveTags = scheme.intent?.objectiveTags ?? scheme.intent?.objective_tags ?? [];
  const objective = objectiveTags.find((tag) => tag.trim().toLowerCase() !== band?.toLowerCase());
  if (band || objective) {
    const parts = [band, objective]
      .filter((value): value is string => Boolean(value?.trim()))
      .map((value) => value.replace(/[_-]+/g, ' ').replace(/^./, (character) => character.toUpperCase()));
    return parts.join(' · ');
  }
  const detail = scheme.summary?.trim()
    || scheme.comparison?.connectionImplication?.trim()
    || scheme.comparison?.supportStrategy?.trim()
    || 'Open this option to review its structural approach.';
  return detail.split(/(?<=[.!?])\s+/)[0] ?? detail;
}

function DesignOptionItem({
  active,
  readOnly,
  optionNumber,
  scheme,
  revision,
  onOpen,
  onIncludedChange,
}: {
  active: boolean;
  readOnly: boolean;
  optionNumber: number;
  scheme: EngineeringScheme;
  revision: DesignOptionRevisionState;
  onOpen: () => void;
  onIncludedChange: (included: boolean) => void;
}) {
  const id = `include-option-${scheme.id.replace(/[^a-z0-9]+/gi, '-')}`;
  return (
    <div className={cn('flex min-w-0 items-center gap-2 px-2 py-2', active && 'bg-muted/50')} aria-current={active ? 'true' : undefined}>
      <Button type="button" onClick={onOpen} variant="ghost" className="h-auto min-w-0 flex-1 justify-start gap-3 px-2 py-2">
        <Badge variant="outline" className="shrink-0">{optionNumber}</Badge>
        <span className="flex min-w-0 flex-1 flex-col items-start gap-0.5 text-left">
          <span className="w-full truncate font-medium" title={scheme.name}>{conciseSchemeTitle(scheme)}</span>
          <span className="w-full truncate text-xs text-muted-foreground" title={conciseSchemeDetail(scheme)}>{conciseSchemeDetail(scheme)}</span>
        </span>
        <ChevronRight data-icon="inline-end" />
      </Button>
      <Field orientation="horizontal" data-disabled={readOnly || undefined} className="w-auto shrink-0">
        <Checkbox
          id={id}
          checked={revision.included}
          disabled={readOnly}
          aria-label={`Include ${scheme.name} for comparison`}
          onCheckedChange={(checked) => onIncludedChange(checked === true)}
        />
        <FieldLabel htmlFor={id} className="sr-only">Include {scheme.name} for comparison</FieldLabel>
      </Field>
    </div>
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
  onCompare,
}: {
  active: ActiveView;
  schemes: EngineeringScheme[];
  batch: DesignOptionBatchState | null;
  batches: DesignOptionBatchState[];
  stage: 'options' | 'analysis';
  busy: boolean;
  onSelectScheme: (id: string) => void;
  onIncludedChange: (id: string, included: boolean) => void;
  onCompare?: () => void;
}) {
  const revisions = batch?.optionRevisions ?? batch?.option_revisions ?? [];
  const entries = schemes.flatMap((scheme, index) => {
    const revision = revisions.find((candidate) => (candidate.optionId ?? candidate.option_id) === scheme.id);
    return revision ? [{ scheme, revision, index }] : [];
  });
  const includedCount = entries.filter(({ revision }) => revision.included).length;
  const historicalBatches = batches.filter((candidate) => candidate.id !== batch?.id);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ScrollArea className="min-h-0 flex-1">
        {entries.length ? (
          <div className="flex flex-col">
            {entries.map(({ scheme, revision, index }) => {
              const optionId = active.kind === 'scheme'
                ? active.id
                : active.kind === 'development'
                  ? active.optionId
                  : null;
              return (
                <div key={scheme.id}>
                  {index > 0 ? <Separator /> : null}
                  <DesignOptionItem
                    active={optionId === scheme.id}
                    readOnly={stage === 'analysis'}
                    optionNumber={index + 1}
                    scheme={scheme}
                    revision={revision}
                    onOpen={() => onSelectScheme(scheme.id)}
                    onIncludedChange={(checked) => !busy && onIncludedChange(scheme.id, checked)}
                  />
                </div>
              );
            })}
          </div>
        ) : (
          <Empty className="h-full">
            <EmptyDescription>No design options are available.</EmptyDescription>
          </Empty>
        )}
      </ScrollArea>

      <Separator />
      <div className="flex shrink-0 items-center justify-between gap-2 p-2">
        <div className="flex min-w-0 items-center gap-2">
          {historicalBatches.length ? (
            <Popover>
              <PopoverTrigger render={<Button variant="outline" size="sm" />}>
                <History data-icon="inline-start" /> History
              </PopoverTrigger>
              <PopoverContent align="start" className="w-80 p-2">
                <div className="px-2 py-1 text-sm font-medium">Previous option batches</div>
                <p className="px-2 pb-2 text-xs text-muted-foreground">Read-only references from earlier Base Model states.</p>
                <ScrollArea className="max-h-72">
                  <div className="flex flex-col gap-2">
                    {[...historicalBatches].reverse().map((historical) => {
                      const historicalRevisions = historical.optionRevisions ?? historical.option_revisions ?? [];
                      return (
                        <Card key={historical.id}>
                          <CardContent className="flex flex-col gap-1 p-3">
                            <div className="flex items-center justify-between gap-2">
                              <span className="truncate font-medium">{historical.id}</span>
                              <Badge variant="outline">{historical.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">{historicalRevisions.length} option revision{historicalRevisions.length === 1 ? '' : 's'} · {historical.generatedAt ?? historical.generated_at}</span>
                          </CardContent>
                        </Card>
                      );
                    })}
                  </div>
                </ScrollArea>
              </PopoverContent>
            </Popover>
          ) : null}
          <span className="truncate text-sm text-muted-foreground">
            {stage === 'analysis' ? `${includedCount} in comparison` : `${includedCount} selected`}
          </span>
        </div>
        {stage === 'options' ? (
          <Button onClick={onCompare} disabled={!includedCount || busy}>
            <Scale data-icon="inline-start" /> Compare {includedCount}
          </Button>
        ) : null}
      </div>
    </div>
  );
}
