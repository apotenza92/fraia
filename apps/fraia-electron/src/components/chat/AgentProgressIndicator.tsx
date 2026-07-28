import { useEffect, useState } from 'react';
import { Progress } from '@/components/ui/progress';

export type AgentProgressStage = { label: string; durationMs: number };

export function useEstimatedAgentProgress(active: boolean, stages: AgentProgressStage[], waitingLabel = 'Model response') {
  const [elapsedMs, setElapsedMs] = useState(0);

  useEffect(() => {
    if (!active) {
      setElapsedMs(0);
      return;
    }
    const startedAt = Date.now();
    setElapsedMs(0);
    const interval = window.setInterval(() => {
      setElapsedMs(Date.now() - startedAt);
    }, 350);
    return () => window.clearInterval(interval);
  }, [active]);

  const totalMs = stages.reduce((sum, stage) => sum + stage.durationMs, 0);
  let cumulativeMs = 0;
  let stageLabel = stages.length ? stages[stages.length - 1].label : 'Working';
  for (const stage of stages) {
    cumulativeMs += stage.durationMs;
    if (elapsedMs <= cumulativeMs) {
      stageLabel = stage.label;
      break;
    }
  }
  if (totalMs > 0 && elapsedMs > totalMs) {
    stageLabel = waitingLabel;
  }
  const percent = active && totalMs > 0
    ? Math.min(92, Math.max(3, Math.round((1 - Math.exp(-elapsedMs / totalMs)) * 92)))
    : 0;
  return { percent, stageLabel };
}

export function EstimatedAgentProgress({ percent, stageLabel }: { percent: number; stageLabel: string }) {
  return (
    <>
      <div className="flex flex-nowrap justify-between gap-4">
        <span className="truncate text-xs text-muted-foreground">{stageLabel}</span>
        <span className="font-mono text-xs text-muted-foreground">~{percent}%</span>
      </div>
      <Progress value={percent} className="mt-1" />
    </>
  );
}
