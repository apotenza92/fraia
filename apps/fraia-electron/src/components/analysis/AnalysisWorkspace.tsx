import { AlertTriangle, Play, SquareStack } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Empty, EmptyDescription } from '@/components/ui/empty';
import { cn } from '@/lib/utils';
import { useState } from 'react';
import type { AgentSession, EngineeringScheme, WorkbenchState } from '../../lib/types';
import { normalizeWorkbenchState, planningRequestFromState, projectDirOf } from '../../lib/defaultProject';

type AnalysisScope = { kind: 'all' } | { kind: 'scheme'; id: string };

function scopeLabel(scope: AnalysisScope, schemes: EngineeringScheme[]) {
  if (scope.kind === 'all') return 'Current project';
  return schemes.find((scheme) => scheme.id === scope.id)?.name ?? 'Selected design option';
}

function sessionMessages(session: AgentSession | undefined) {
  return (session?.messages ?? []).filter((message) => !['deterministic', 'local'].includes(message.mode ?? ''));
}

function messageReplies(message: { suggestedReplies?: string[]; suggested_replies?: string[] }) {
  return message.suggestedReplies ?? message.suggested_replies ?? [];
}

function schemeSession(state: WorkbenchState | null, schemeId: string) {
  const sessions = state?.agentState?.sessions ?? state?.agent_state?.sessions ?? [];
  return sessions.find((session) => session.surface === `scheme:${schemeId}`);
}

function pendingDecisionForScheme(state: WorkbenchState | null, scheme: EngineeringScheme) {
  if (scheme.status === 'needs_decision') {
    return `${scheme.name} still has unresolved design-option decisions.`;
  }
  const session = schemeSession(state, scheme.id);
  const currentQuestion = session?.currentQuestion ?? session?.current_question;
  const questionOptions = currentQuestion?.options ?? [];
  if (currentQuestion && questionOptions.length) {
    return currentQuestion.prompt || `${scheme.name} has an unanswered design-option question.`;
  }
  const messages = sessionMessages(session);
  const lastUserIndex = [...messages].map((message, index) => ({ message, index })).reverse().find(({ message }) => message.author === 'user')?.index ?? -1;
  const pendingAssistant = [...messages]
    .map((message, index) => ({ message, index }))
    .reverse()
    .find(({ message, index }) => message.author === 'assistant' && index > lastUserIndex && messageReplies(message).length > 0);
  if (!pendingAssistant) return null;
  const text = pendingAssistant.message.text.trim().split('\n').find(Boolean) ?? `${scheme.name} has an unanswered design-option prompt.`;
  return text.length > 180 ? `${text.slice(0, 177)}...` : text;
}

export function AnalysisWorkspace({
  state,
  schemes,
  onState,
  onSelectScheme,
  onRunComplete,
}: {
  state: WorkbenchState | null;
  schemes: EngineeringScheme[];
  onState: (s: WorkbenchState) => void;
  onSelectScheme: (id: string) => void;
  onRunComplete?: () => void;
}) {
  const [scope, setScope] = useState<AnalysisScope>({ kind: 'all' });
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState('Run Base Model readiness checks or analyse realised design-option candidate sections.');
  const schemeCount = schemes.length;
  const blockersByScheme = new Map(schemes.flatMap((scheme) => {
    const blocker = pendingDecisionForScheme(state, scheme);
    return blocker ? [[scheme.id, blocker] as const] : [];
  }));
  const scopeBlockers = (nextScope: AnalysisScope) => nextScope.kind === 'all'
    ? schemes.filter((scheme) => blockersByScheme.has(scheme.id)).map((scheme) => ({ scheme, reason: blockersByScheme.get(scheme.id)! }))
    : schemes
        .filter((scheme) => scheme.id === nextScope.id && blockersByScheme.has(scheme.id))
        .map((scheme) => ({ scheme, reason: blockersByScheme.get(scheme.id)! }));

  async function runSupportedAnalysis(nextScope = scope) {
    const blockers = scopeBlockers(nextScope);
    if (blockers.length) {
      const label = scopeLabel(nextScope, schemes);
      setStatus(`${label} cannot run yet: ${blockers.map(({ scheme }) => scheme.name).join(', ')} ${blockers.length === 1 ? 'has' : 'have'} unresolved design-option decisions.`);
      return;
    }
    setBusy(true);
    setStatus(nextScope.kind === 'all' && schemeCount > 0 ? 'Analysing active design options and candidate sections.' : nextScope.kind === 'scheme' ? 'Analysing selected design-option candidate sections.' : 'Checking current-project analysis readiness.');
    try {
      const response = schemeCount > 0
        ? await window.fraia.analyseDesignOptions({
            projectDir: projectDirOf(state),
            scope: nextScope.kind === 'scheme'
              ? { kind: 'selected_design_options', optionIds: [nextScope.id] }
              : { kind: 'all_active_design_options', optionIds: [] },
            candidatePolicy: 'all_candidates',
            checkProfile: 'preliminary_conservative_steel',
          })
        : await window.fraia.analysePlanning(planningRequestFromState(state));
      const nextState = normalizeWorkbenchState(response);
      if (nextState) onState(nextState);
      setStatus(response?.message ?? 'Analysis request completed.');
      onRunComplete?.();
    } catch (error: any) {
      setStatus(error?.message ?? String(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="h-full flex-1 overflow-auto">
      <div className="mx-auto flex max-w-5xl flex-col gap-4 p-4">
          <header>
            <h1 className="text-2xl font-semibold">Analysis</h1>
            <p className="text-sm text-muted-foreground">Analyse realised design options, candidate sections, and preliminary stress checks.</p>
          </header>

          <Card>
            <CardContent className="flex flex-nowrap items-start justify-between gap-4">
              <div className="flex flex-col gap-1">
                <div className="flex items-center gap-2 font-semibold">
                  <SquareStack />
                  <span>{schemeCount ? 'All Design Options' : 'Current Base Model'}</span>
                </div>
                <p className="text-sm text-muted-foreground">
                  {schemeCount
                    ? 'Runs CalculiX for each active design option and checks candidate sections with a conservative preliminary stress screen.'
                    : 'Check whether the current Base Model can run through the supported solver path. Design options can be added later when the agent has justified intents.'}
                </p>
                {blockersByScheme.size > 0 && schemeCount > 0 && (
                  <Alert>
                    <AlertTriangle />
                    <AlertDescription>Run all is blocked until every design option has resolved its pending chat decision.</AlertDescription>
                  </Alert>
                )}
              </div>
              <Button onClick={() => { const next = { kind: 'all' } as const; setScope(next); runSupportedAnalysis(next); }} disabled={busy || (schemeCount > 0 && blockersByScheme.size > 0)}>
                <Play data-icon="inline-start" />
                {busy && scope.kind === 'all' ? 'Running' : schemeCount ? 'Analyse all options' : 'Check readiness'}
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="flex flex-col gap-0">
              {schemes.map((scheme) => {
                const selected = scope.kind === 'scheme' && scope.id === scheme.id;
                const nextScope = { kind: 'scheme', id: scheme.id } as const;
                const blocker = blockersByScheme.get(scheme.id);
                return (
                  <div
                    key={scheme.id}
                    className={cn('border-b p-3 last:border-b-0', selected && 'bg-accent text-accent-foreground')}
                  >
                    <div className="flex flex-nowrap items-center justify-between gap-4">
                    <div className="min-w-0 flex-1">
                      <Button onClick={() => setScope(nextScope)} variant="link" className="h-auto max-w-full justify-start p-0">
                        {scheme.name}
                      </Button>
                      <p className="truncate text-sm text-muted-foreground">{scheme.comparison.supportStrategy} - {scheme.comparison.connectionImplication}</p>
                      {blocker && (
                        <Alert className="mt-2">
                          <AlertTriangle />
                          <AlertDescription>{blocker}</AlertDescription>
                        </Alert>
                      )}
                    </div>
                    <div className="flex flex-nowrap gap-2">
                      <Button onClick={() => { setScope(nextScope); runSupportedAnalysis(nextScope); }} disabled={busy || Boolean(blocker)} size="sm">Analyse</Button>
                      <Button onClick={() => onSelectScheme(scheme.id)} size="sm">Open chat</Button>
                    </div>
                    </div>
                  </div>
                );
              })}
              {!schemes.length && (
                <Empty className="min-h-32">
                  <EmptyDescription>No design options are available yet.</EmptyDescription>
                </Empty>
              )}
            </CardContent>
          </Card>
      </div>
    </main>
  );
}
