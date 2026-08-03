import { CheckCircle2, FileSearch, History, Route } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import type { DevelopmentPathState, EngineeringScheme } from '../../lib/types';

export function DevelopmentPanel({ scheme, path, onOpenEvidence, onBackToComparison }: { scheme: EngineeringScheme; path: DevelopmentPathState; onOpenEvidence: () => void; onBackToComparison: () => void }) {
  const analysis = scheme.analysisSummary;
  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col gap-3 p-3">
        <div>
          <div className="flex items-center gap-2"><Badge>Option work</Badge><span className="text-xs text-muted-foreground">Preserved analysis path</span></div>
          <h2 className="mt-2 text-lg font-semibold">{scheme.name}</h2>
          <p className="mt-1 text-sm text-muted-foreground">Continue working from this analysed option without overwriting the Base Model or comparison evidence.</p>
        </div>

        <Card>
          <CardHeader><CardTitle className="flex items-center gap-2"><Route /> Source decision</CardTitle></CardHeader>
          <CardContent className="flex flex-col gap-2">
            <div className="flex items-center gap-2"><CheckCircle2 className="size-4 text-primary" /><span>Current preliminary analysis attached</span></div>
            <p className="text-muted-foreground">Maximum utilisation {analysis?.maxUtilization?.toFixed(2) ?? '—'} · Deflection {analysis?.maxDeflectionMm?.toFixed(1) ?? '—'} mm</p>
            <p className="text-xs text-muted-foreground">Path {path.id}</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader><CardTitle>Available work</CardTitle></CardHeader>
          <CardContent className="flex flex-col gap-2">
            <p>Review the selected sections, grouping, preliminary utilisation, displacement, and governing evidence attached to this path.</p>
            <p className="text-muted-foreground">Code, connection, foundation, and construction-document checks are not yet available and are not implied by this path.</p>
          </CardContent>
        </Card>

        <Button onClick={onOpenEvidence}><FileSearch data-icon="inline-start" /> Engineering evidence</Button>
        <Button onClick={onBackToComparison} variant="outline"><History data-icon="inline-start" /> Review comparison</Button>
        <p className="text-xs text-muted-foreground">Working on another option creates or reopens a separate preserved path.</p>
      </div>
    </ScrollArea>
  );
}
