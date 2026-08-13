import { Alert, AlertDescription } from '@/components/ui/alert';
import type { WorkbenchState } from '@/lib/types';
import { DocumentTabBar, documentTabTriggerId, type DocumentTab } from '../domain-ui/DocumentTabBar';
import { AppMenuBar } from './AppMenuBar';
import { ConversationWorkspace as ConversationWorkspaceSurface } from '../conversation/ConversationWorkspace';
import { APP_HEADER_HEIGHT, CHROME } from './chromeMetrics';

export function AppShell({
  state,
  documentTabs,
  activeDocumentId,
  onDocumentSelect,
  onDocumentClose,
  onDocumentReorder,
  onOpenDocument,
  onNewBlankModel,
  documentActionPending,
  documentError,
}: {
  state: WorkbenchState;
  onState: (nextState: WorkbenchState) => void;
  documentTabs: DocumentTab[];
  activeDocumentId: string;
  onDocumentSelect: (id: string) => void;
  onDocumentClose: (id: string) => void;
  onDocumentReorder: (orderedIds: string[]) => void;
  onOpenDocument: () => void;
  onNewBlankModel: () => void;
  documentActionPending: boolean;
  documentError?: string | null;
}) {
  return (
    <div data-testid="conversation-workspace-shell" className="grid h-screen w-screen grid-rows-[auto_minmax(0,1fr)] overflow-hidden bg-background text-foreground">
      <header style={{ height: APP_HEADER_HEIGHT }}>
        <AppMenuBar />
        <div className="shrink-0" style={{ height: CHROME.tabHeight }}>
          <DocumentTabBar
            tabs={documentTabs}
            value={activeDocumentId}
            panelId="fraia-conversation-panel"
            onValueChange={onDocumentSelect}
            onClose={onDocumentClose}
            onReorder={onDocumentReorder}
            onOpen={onOpenDocument}
            openDisabled={documentActionPending}
            onNewBlankModel={onNewBlankModel}
            newBlankModelDisabled={documentActionPending}
          />
        </div>
      </header>
      <main className="min-h-0 min-w-0">
        <h1 className="sr-only">Fraia structural design workspace</h1>
        <section
          id="fraia-conversation-panel"
          role="tabpanel"
          aria-labelledby={documentTabTriggerId(activeDocumentId)}
          className="flex min-h-0 min-w-0 flex-col"
        >
          {documentError ? (
            <div className="shrink-0 px-4 pt-3">
              <Alert variant="destructive"><AlertDescription>{documentError}</AlertDescription></Alert>
            </div>
          ) : null}
          <ConversationWorkspaceSurface state={state} />
        </section>
      </main>
    </div>
  );
}
