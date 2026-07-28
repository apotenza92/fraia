import { useState } from 'react';
import { Play } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import type { WorkbenchState } from '../../lib/types';
import { normalizeWorkbenchState, planningRequestFromState } from '../../lib/defaultProject';

export function SolvePanel({
  state,
  onState,
}: {
  state: WorkbenchState | null;
  onState: (s: WorkbenchState) => void;
}) {
  const [status, setStatus] = useState('Ready to run a pre-solve readiness check.');
  const [busy, setBusy] = useState(false);
  const readiness = state?.analysisReadiness || state?.analysis_readiness;

  async function solve() {
    setBusy(true);
    try {
      const res = await window.fraia.analysePlanning(planningRequestFromState(state));
      const nextState = normalizeWorkbenchState(res);
      if (nextState) onState(nextState);
      setStatus('Analysis request completed. Review diagnostics and design options below.');
    } catch (error: any) {
      setStatus(`Solve/backend response: ${error.message}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="h-full overflow-auto">
      <div className="flex flex-col gap-4 p-6">
      <div>
        <h1 className="text-2xl font-semibold">Solve</h1>
        <p className="text-sm text-muted-foreground">Solve panel owns readiness, run feedback, and result summaries.</p>
      </div>
      <Alert>
        <AlertDescription>{readiness?.summary || status}</AlertDescription>
      </Alert>
      <Button onClick={solve} disabled={busy} className="w-full">
        <Play />
        {busy ? 'Running...' : 'Run solve readiness'}
      </Button>
      <Card>
        <CardHeader>
          <CardTitle>Current model</CardTitle>
        </CardHeader>
        <CardContent className="grid grid-cols-2 gap-3">
          <Card>
            <CardHeader>
              <CardTitle>{state?.scene?.nodes?.length ?? 0}</CardTitle>
              <CardDescription>Nodes</CardDescription>
            </CardHeader>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>{state?.scene?.members?.length ?? 0}</CardTitle>
              <CardDescription>Members</CardDescription>
            </CardHeader>
          </Card>
        </CardContent>
      </Card>
      </div>
    </div>
  );
}
