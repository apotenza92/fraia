import { useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { FileBox, FilePlus2, FolderOpen } from 'lucide-react';
import { AppShell } from './components/layout/AppShell';
import { AppMenuBar } from './components/layout/AppMenuBar';
import { DocumentTabBar } from './components/domain-ui/DocumentTabBar';
import { APP_HEADER_HEIGHT, CHROME } from './components/layout/chromeMetrics';
import { normalizeWorkbenchState } from './lib/defaultProject';
import {
  projectDocumentFromState,
  reorderProjectDocuments,
  upsertProjectDocument,
  type ProjectDocument,
} from './lib/projectDocuments';
import { useSystemTheme } from './lib/theme';
import type { WorkbenchState } from './lib/types';

function EmptyWorkspaceShell({
  error,
  pending,
  onOpen,
  onNew,
}: {
  error: string | null;
  pending: boolean;
  onOpen: () => void;
  onNew: () => void;
}) {
  return (
    <div data-testid="conversation-workspace-shell" className="grid h-screen w-screen grid-rows-[auto_minmax(0,1fr)] overflow-hidden bg-background text-foreground">
      <header style={{ height: APP_HEADER_HEIGHT }}>
        <AppMenuBar />
        <div className="shrink-0" style={{ height: CHROME.tabHeight }}>
          <DocumentTabBar
            tabs={[]}
            value=""
            panelId="fraia-empty-workspace"
            onValueChange={() => {}}
            onClose={() => {}}
            onReorder={() => {}}
            onOpen={onOpen}
            openDisabled={pending}
            onNewBlankModel={onNew}
            newBlankModelDisabled={pending}
          />
        </div>
      </header>
      <main id="fraia-empty-workspace" className="flex min-h-0 min-w-0 flex-col p-6">
        {error ? (
          <Alert variant="destructive" role="alert" className="mx-auto w-full max-w-xl">
            <AlertTitle>Fraia could not open the model</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        <Empty data-testid="empty-workspace">
          <EmptyHeader>
            <EmptyMedia variant="icon"><FileBox /></EmptyMedia>
            <EmptyTitle>No models are open</EmptyTitle>
            <EmptyDescription>Open an existing Fraia model or create a blank model to start.</EmptyDescription>
          </EmptyHeader>
          <EmptyContent className="flex-row justify-center">
            <Button variant="outline" disabled={pending} onClick={onOpen}><FolderOpen data-icon="inline-start" />Open model</Button>
            <Button disabled={pending} onClick={onNew}><FilePlus2 data-icon="inline-start" />New blank model</Button>
          </EmptyContent>
        </Empty>
      </main>
    </div>
  );
}

export default function App() {
  const [documents, setDocuments] = useState<ProjectDocument[]>([]);
  const [activeDocumentId, setActiveDocumentId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [documentActionPending, setDocumentActionPending] = useState(false);
  useSystemTheme();
  const activeDocument = documents.find((document) => document.id === activeDocumentId) ?? documents[0] ?? null;

  function updateActiveDocument(nextState: WorkbenchState) {
    const nextDocument = projectDocumentFromState(nextState);
    setDocuments((current) => current.map((document) => (
      document.id === activeDocument?.id ? nextDocument : document
    )));
    if (activeDocument?.id !== nextDocument.id) setActiveDocumentId(nextDocument.id);
  }

  async function openProjectDocument() {
    if (documentActionPending) return;
    setError(null);
    setDocumentActionPending(true);
    try {
      const projectDir = await window.fraia.pickProjectFile();
      if (!projectDir) return;
      const existing = documents.find((document) => document.projectDir === projectDir);
      if (existing) {
        setActiveDocumentId(existing.id);
        return;
      }
      const nextState = normalizeWorkbenchState(await window.fraia.openProject({ projectDir }));
      if (!nextState) throw new Error('Fraia did not return the selected model.');
      const nextDocument = projectDocumentFromState(nextState);
      setDocuments((current) => upsertProjectDocument(current, nextDocument));
      setActiveDocumentId(nextDocument.id);
    } catch (caught: any) {
      setError(caught?.message || 'Could not open the selected Fraia model.');
    } finally {
      setDocumentActionPending(false);
    }
  }

  async function createBlankModel() {
    if (documentActionPending) return;
    setError(null);
    setDocumentActionPending(true);
    try {
      const projectDir = await window.fraia.createUntitledProject();
      const nextState = normalizeWorkbenchState(await window.fraia.createProject({ projectDir, name: 'Untitled Model' }));
      if (!nextState) throw new Error('Fraia did not return the new blank model.');
      const nextDocument = projectDocumentFromState(nextState);
      setDocuments((current) => upsertProjectDocument(current, nextDocument));
      setActiveDocumentId(nextDocument.id);
    } catch (caught: any) {
      setError(caught?.message || 'Could not create a new blank model.');
    } finally {
      setDocumentActionPending(false);
    }
  }

  function closeDocument(documentId: string) {
    const closingIndex = documents.findIndex((document) => document.id === documentId);
    if (closingIndex < 0) return;
    const remaining = documents.filter((document) => document.id !== documentId);
    if (activeDocumentId === documentId) {
      setActiveDocumentId(remaining[closingIndex]?.id ?? remaining[closingIndex - 1]?.id ?? null);
    }
    setDocuments(remaining);
  }

  if (!activeDocument) {
    return (
      <EmptyWorkspaceShell
        error={error}
        pending={documentActionPending}
        onOpen={() => { void openProjectDocument(); }}
        onNew={() => { void createBlankModel(); }}
      />
    );
  }

  const documentTabs = documents.map((document) => ({
    id: document.id,
    label: document.label,
    closable: true,
    reorderable: true,
  }));

  return (
    <AppShell
      key={activeDocument.id}
      state={activeDocument.state}
      onState={updateActiveDocument}
      documentTabs={documentTabs}
      activeDocumentId={activeDocument.id}
      onDocumentSelect={setActiveDocumentId}
      onDocumentClose={closeDocument}
      onDocumentReorder={(orderedIds) => setDocuments((current) => reorderProjectDocuments(current, orderedIds))}
      onOpenDocument={() => { void openProjectDocument(); }}
      onNewBlankModel={() => { void createBlankModel(); }}
      documentActionPending={documentActionPending}
      documentError={error}
    />
  );
}
