import { AlertTriangle, Bot, CheckCircle2, FileSearch, Gauge, GitCommitHorizontal, Scale, Sparkles } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { DesignOptionRevisionState, DevelopmentPathState, EngineeringScheme, WorkbenchState } from '../../lib/types';
import { activeBatchFrom, latestComparisonFrom, optionIdForPath, optionRevisions } from '../../lib/designOptionDecisions';
import { SchemeChatPanel } from '../panels/SchemeChatPanel';

function metric(value: number | null | undefined, suffix: string, digits = 1) {
  return value == null || !Number.isFinite(value) ? '—' : `${value.toFixed(digits)} ${suffix}`;
}

function optionAnalysisStatus(revision: DesignOptionRevisionState | null) {
  const status = revision?.analysisStatus ?? revision?.analysis_status ?? 'not_run';
  if (status === 'current') return { label: 'Current evidence', variant: 'default' as const };
  if (status === 'failed') return { label: 'Analysis needs attention', variant: 'destructive' as const };
  if (status === 'stale') return { label: 'Evidence outdated', variant: 'outline' as const };
  return { label: 'Not analysed', variant: 'secondary' as const };
}

export function DesignOptionInspector({
  state,
  scheme,
  revision,
  stage,
  comparisonCurrent,
  developmentPaths,
  activePathId,
  onState,
  onDevelop,
  onOpenPath,
  onOpenEvidence,
}: {
  state: WorkbenchState | null;
  scheme: EngineeringScheme;
  revision: DesignOptionRevisionState | null;
  stage: 'options' | 'analysis';
  comparisonCurrent: boolean;
  developmentPaths: DevelopmentPathState[];
  activePathId: string | null;
  onState: (state: WorkbenchState) => void;
  onDevelop: () => void;
  onOpenPath: (optionId: string) => void;
  onOpenEvidence: () => void;
}) {
  const analysis = scheme.analysisSummary;
  const current = (revision?.analysisStatus ?? revision?.analysis_status) === 'current';
  const comparison = latestComparisonFrom(state);
  const recommendedId = comparison?.recommendedOptionId ?? comparison?.recommended_option_id;
  const recommended = comparisonCurrent && recommendedId === scheme.id;
  const status = optionAnalysisStatus(revision);
  const intent = scheme.intent;
  const sectionPolicy = intent?.sectionFamilyPolicy ?? intent?.section_family_policy ?? 'Review allowed section families';

  return (
    <div className="flex h-full min-h-0 flex-col border-l bg-background">
      <div className="shrink-0 border-b p-3">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">Inspected option</div>
            <h2 className="truncate text-lg font-semibold" title={scheme.name}>{scheme.name}</h2>
          </div>
          <Badge variant={status.variant}>{status.label}</Badge>
        </div>
        {stage === 'analysis' && recommended ? (
          <div className="mt-2 flex items-center gap-2 text-sm font-medium text-primary">
            <Sparkles className="size-4" /> Fraia recommendation
          </div>
        ) : null}
      </div>

      <Tabs defaultValue="summary" className="flex min-h-0 flex-1 flex-col">
        <TabsList variant="line" className="mx-3 mt-2 shrink-0 justify-start" activateOnFocus>
          <TabsTrigger value="summary">Summary</TabsTrigger>
          <TabsTrigger value="assistant"><Bot /> Assistant</TabsTrigger>
        </TabsList>
        <TabsContent value="summary" className="min-h-0 flex-1">
          <ScrollArea className="h-full">
            <div className="flex flex-col gap-3 p-3">
              {stage === 'analysis' && recommended && comparison ? (
                <Alert>
                  <Sparkles />
                  <AlertTitle>Recommended for the current objective</AlertTitle>
                  <AlertDescription>{comparison.explanation}</AlertDescription>
                </Alert>
              ) : null}

              <Card>
                <CardHeader><CardTitle className="text-sm">What this option tests</CardTitle></CardHeader>
                <CardContent className="text-sm text-muted-foreground">{scheme.summary}</CardContent>
              </Card>

              {stage === 'analysis' ? <div className="grid grid-cols-2 gap-2">
                <Card><CardContent className="flex flex-col gap-1"><Scale className="size-4 text-muted-foreground" /><span className="text-xs text-muted-foreground">Estimated mass</span><strong>{metric(scheme.approximateMassKg, 'kg', 0)}</strong></CardContent></Card>
                <Card><CardContent className="flex flex-col gap-1"><Gauge className="size-4 text-muted-foreground" /><span className="text-xs text-muted-foreground">Max utilisation</span><strong>{analysis?.maxUtilization == null ? '—' : analysis.maxUtilization.toFixed(2)}</strong></CardContent></Card>
                <Card><CardContent className="flex flex-col gap-1"><GitCommitHorizontal className="size-4 text-muted-foreground" /><span className="text-xs text-muted-foreground">Deflection</span><strong>{metric(analysis?.maxDeflectionMm, 'mm')}</strong></CardContent></Card>
                <Card><CardContent className="flex flex-col gap-1"><CheckCircle2 className="size-4 text-muted-foreground" /><span className="text-xs text-muted-foreground">Stress</span><strong>{metric(analysis?.maxStressMpa, 'MPa')}</strong></CardContent></Card>
              </div> : null}

              <Card>
                <CardHeader><CardTitle className="text-sm">Key differences and assumptions</CardTitle></CardHeader>
                <CardContent className="flex flex-col gap-3 text-sm">
                  <div><div className="font-medium">Support strategy</div><p className="text-muted-foreground">{scheme.comparison.supportStrategy}</p></div>
                  <div><div className="font-medium">Connection and load path</div><p className="text-muted-foreground">{scheme.comparison.connectionImplication}</p></div>
                  <div><div className="font-medium">Section policy</div><p className="text-muted-foreground">{sectionPolicy}</p></div>
                  {scheme.assumptions.slice(0, 5).map((assumption) => <p key={assumption} className="border-l-2 pl-2 text-muted-foreground">{assumption}</p>)}
                </CardContent>
              </Card>

              {stage === 'analysis' && !current ? (
                <Alert>
                  <AlertTriangle />
                  <AlertTitle>Current preliminary evidence required</AlertTitle>
                  <AlertDescription>Include this option and run Analyse options before opening a preserved work path.</AlertDescription>
                </Alert>
              ) : null}

              {stage === 'analysis' ? (
                <Card>
                  <CardHeader><CardTitle className="text-sm">Analysis outputs</CardTitle></CardHeader>
                  <CardContent className="flex flex-col gap-2 text-sm">
                    <p className="text-muted-foreground">
                      {current
                        ? 'A preliminary option summary and its immutable solver evidence are available.'
                        : 'Outputs are unavailable until this included revision has current successful evidence.'}
                    </p>
                    <Button onClick={onOpenEvidence} disabled={!current} variant="outline" className="w-full">
                      <FileSearch data-icon="inline-start" /> Engineering evidence
                    </Button>
                  </CardContent>
                </Card>
              ) : null}

              {stage === 'analysis' && developmentPaths.length ? (
                <Card>
                  <CardHeader><CardTitle className="text-sm">Preserved option paths</CardTitle></CardHeader>
                  <CardContent className="flex flex-col gap-2">
                    {developmentPaths.map((path) => {
                      const optionId = optionIdForPath(path);
                      const option = (state?.designSchemes ?? state?.design_schemes ?? []).find((candidate: any) => candidate.id === optionId);
                      const pathRevisionId = path.optionRevisionId ?? path.option_revision_id;
                      const sourceRunId = path.sourceAnalysisRunId ?? path.source_analysis_run_id;
                      const currentRevision = optionRevisions(activeBatchFrom(state)).find((candidate) => (
                        (candidate.revisionId ?? candidate.revision_id ?? candidate.optionId ?? candidate.option_id) === pathRevisionId
                      ));
                      const currentRunId = currentRevision?.latestAnalysisRunId ?? currentRevision?.latest_analysis_run_id;
                      const pathCanReopen = Boolean(
                        currentRevision?.included
                          && (currentRevision.analysisStatus ?? currentRevision.analysis_status) === 'current'
                          && currentRunId,
                      );
                      const pathIsCurrent = pathCanReopen && path.id === activePathId && sourceRunId === currentRunId;
                      return (
                        <div key={path.id} className="flex flex-col gap-1">
                          <Button
                            onClick={() => onOpenPath(optionId)}
                            disabled={!pathCanReopen}
                            variant={pathIsCurrent ? 'secondary' : 'outline'}
                            className="w-full justify-start"
                          >
                            {pathIsCurrent ? 'Current: ' : pathCanReopen ? 'Open: ' : 'Historical: '}{option?.name ?? optionId}
                          </Button>
                          {!pathCanReopen ? <span className="px-2 text-xs text-muted-foreground">Reference path from an earlier or unavailable option revision.</span> : null}
                        </div>
                      );
                    })}
                  </CardContent>
                </Card>
              ) : null}

              {stage === 'analysis' ? (
                <Button onClick={onDevelop} disabled={!current || revision?.included === false} className="w-full">
                  Work on this option
                </Button>
              ) : null}
              <p className="text-xs text-muted-foreground">
                {stage === 'analysis'
                  ? 'Preliminary evidence supports comparison only; it is not a full code-compliant design check.'
                  : 'Shortlisting changes which options move forward; it does not run analysis or select a preferred option.'}
              </p>
            </div>
          </ScrollArea>
        </TabsContent>
        <TabsContent value="assistant" className="min-h-0 flex-1">
          <SchemeChatPanel state={state} scheme={scheme} surface={`scheme:${scheme.id}`} onState={onState} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
