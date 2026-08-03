import type { ReactNode } from 'react';
import { Separator } from '@/components/ui/separator';
import { CHROME } from './chromeMetrics';

export function ModelWorkspaceChrome({
  title,
  leading,
  panelWidth,
  panelOpen,
  children,
  trailing,
}: {
  title: string;
  leading?: ReactNode;
  panelWidth: number;
  panelOpen: boolean;
  children?: ReactNode;
  trailing?: ReactNode;
}) {
  return (
    <div className="min-w-0 shrink-0">
      <div className="flex min-w-0 flex-nowrap" style={{ height: CHROME.workspaceToolbarHeight }}>
        <div
          className="relative flex h-full flex-nowrap items-center justify-center px-3"
          style={{ width: panelOpen ? panelWidth : 0, minWidth: panelOpen ? panelWidth : 0, maxWidth: panelOpen ? panelWidth : 0 }}
        >
          {leading ? <div className="absolute inset-y-0 left-2 flex items-center">{leading}</div> : null}
          {panelOpen ? <h1 id="fraia-workflow-stage-heading" tabIndex={-1} className="truncate text-center font-semibold">{title}</h1> : null}
        </div>
        <div className="relative flex h-full min-w-0 flex-1 flex-nowrap items-center justify-center px-3">
          {!panelOpen ? (
            <div className="absolute inset-y-0 left-3 flex items-center">
              <h1 id="fraia-workflow-stage-heading" tabIndex={-1} className="truncate font-semibold">{title}</h1>
            </div>
          ) : null}
          <div className="flex min-w-0 flex-nowrap items-center justify-center gap-1">{children}</div>
          {trailing ? <div className="absolute inset-y-0 right-3 flex items-center gap-2">{trailing}</div> : null}
        </div>
      </div>
      <Separator />
    </div>
  );
}
