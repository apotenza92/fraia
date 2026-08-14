import { useState } from 'react';
import { BookOpen, Library } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import type { WorkbenchState } from '@/lib/types';
import { DocumentTabBar, documentTabTriggerId, type DocumentTab } from '../domain-ui/DocumentTabBar';
import { AppMenuBar } from './AppMenuBar';
import { ConversationWorkspace as ConversationWorkspaceSurface } from '../conversation/ConversationWorkspace';
import { APP_HEADER_HEIGHT, CHROME } from './chromeMetrics';
import { ResourceLibrarySheet } from '../sources/ResourceLibrarySheet';
import { AiProvidersDialog } from '../ai/AiProvidersDialog';

export function AppShell({
  state,
  onState,
  documentTabs,
  activeDocumentId,
  onDocumentSelect,
  onDocumentClose,
  onDocumentReorder,
  onOpenDocument,
  onNewBlankModel,
  onRenameProject,
  onRenameDesign,
  onDeleteDesign,
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
  onRenameProject: () => void;
  onRenameDesign: () => void;
  onDeleteDesign: () => void;
  documentActionPending: boolean;
  documentError?: string | null;
}) {
  const [resourceView, setResourceView] = useState<'sources' | 'shelf' | null>(null);
  const [connectionOpen, setConnectionOpen] = useState(false);
  const projectDir = state.overview?.projectRootDir ?? state.overview?.project_root_dir ?? state.overview?.projectDir ?? state.overview?.project_dir ?? '';
  const projectName = state.overview?.projectName ?? state.overview?.project_name ?? 'Untitled Project';
  const projectId = state.overview?.projectId ?? state.overview?.project_id ?? '';
  const designId = state.overview?.designId ?? state.overview?.design_id ?? state.overview?.documentId ?? state.overview?.document_id ?? '';
  const designName = state.overview?.designName ?? state.overview?.design_name ?? 'Design 1';
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
            onRenameProject={onRenameProject}
            onRenameDesign={onRenameDesign}
            onDeleteDesign={onDeleteDesign}
            onConnection={() => setConnectionOpen(true)}
            newBlankModelDisabled={documentActionPending}
          />
        </div>
      </header>
      <main className="flex h-full min-h-0 min-w-0 flex-col">
        <h1 className="sr-only">Fraia structural design workspace</h1>
        <div className="flex shrink-0 flex-wrap items-center justify-end gap-1 px-3 py-2" data-purpose="open-design-inputs">
          <Button variant="ghost" size="sm" onClick={() => setResourceView('sources')}><Library data-icon="inline-start" />Files</Button>
          <Button variant="outline" size="sm" aria-label={`${designName} references`} onClick={() => setResourceView('shelf')}><BookOpen data-icon="inline-start" />References</Button>
        </div>
        <Separator />
        <section
          id="fraia-conversation-panel"
          role="tabpanel"
          aria-labelledby={documentTabTriggerId(activeDocumentId)}
          className="flex min-h-0 min-w-0 flex-1 flex-col"
        >
          {documentError ? (
            <div className="shrink-0 px-4 pt-3">
              <Alert variant="destructive"><AlertDescription>{documentError}</AlertDescription></Alert>
            </div>
          ) : null}
          <ConversationWorkspaceSurface state={state} onState={onState} />
        </section>
        <ResourceLibrarySheet
          open={resourceView !== null}
          initialView={resourceView ?? 'sources'}
          projectDir={projectDir}
          projectId={projectId}
          projectName={projectName}
          designId={designId}
          designName={designName}
          onOpenChange={(open) => { if (!open) setResourceView(null); }}
        />
        <AiProvidersDialog open={connectionOpen} onOpenChange={setConnectionOpen} />
      </main>
    </div>
  );
}
