import { useEffect, useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { FileBox, FilePlus2, FolderOpen } from 'lucide-react';
import { AppShell } from './components/layout/AppShell';
import { AppMenuBar } from './components/layout/AppMenuBar';
import { DocumentTabBar } from './components/domain-ui/DocumentTabBar';
import { FirstSaveDialog, type FirstSaveNames } from './components/project/FirstSaveDialog';
import { NameDialog } from './components/project/NameDialog';
import { APP_HEADER_HEIGHT, CHROME } from './components/layout/chromeMetrics';
import { normalizeWorkbenchState } from './lib/defaultProject';
import {
  projectDocumentFromState,
  preserveProjectIdentity,
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
  const [firstSaveOpen, setFirstSaveOpen] = useState(false);
  const [firstSaveError, setFirstSaveError] = useState<string | null>(null);
  const [nameAction, setNameAction] = useState<'create-design' | 'rename-design' | 'rename-project' | null>(null);
  const [nameError, setNameError] = useState<string | null>(null);
  useSystemTheme();
  const activeDocument = documents.find((document) => document.id === activeDocumentId) ?? documents[0] ?? null;

  function updateActiveDocument(nextState: WorkbenchState) {
    if (!activeDocument) return;
    const nextDocument = projectDocumentFromState(preserveProjectIdentity(nextState, activeDocument));
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
      const response = await window.fraia.openProject({ projectDir });
      const designStates = response?.designStates ?? [response?.state ?? response];
      const nextStates = designStates.map(normalizeWorkbenchState).filter(Boolean) as WorkbenchState[];
      const nextState = nextStates[0];
      if (!nextState) throw new Error('Fraia did not return the selected model.');
      const nextDocuments = nextStates.map(projectDocumentFromState);
      setDocuments((current) => nextDocuments.reduce(upsertProjectDocument, current));
      setActiveDocumentId(nextDocuments[0].id);
    } catch (caught: any) {
      setError(caught?.message || 'Could not open the selected Fraia model.');
    } finally {
      setDocumentActionPending(false);
    }
  }

  async function submitNameAction(name: string) {
    if (!activeDocument || !nameAction || documentActionPending) return;
    setNameError(null);
    setDocumentActionPending(true);
    try {
      if (nameAction === 'create-design') {
        const nextState = normalizeWorkbenchState(await window.fraia.createDesign({
          projectDir: activeDocument.projectRootDir,
          projectId: activeDocument.projectId,
          designName: name,
        }));
        if (!nextState) throw new Error('Fraia did not return the new design.');
        const nextDocument = projectDocumentFromState(nextState);
        setDocuments((current) => upsertProjectDocument(current, nextDocument));
        setActiveDocumentId(nextDocument.id);
      } else {
        const nextState = normalizeWorkbenchState(await window.fraia.renameDesign({
          projectDir: activeDocument.projectRootDir,
          projectId: activeDocument.projectId,
          projectName: nameAction === 'rename-project' ? name : activeDocument.projectName,
          designId: activeDocument.designId,
          designName: nameAction === 'rename-design' ? name : activeDocument.designName,
        }));
        if (!nextState) throw new Error('Fraia did not return the renamed design.');
        const nextDocument = projectDocumentFromState(nextState);
        setDocuments((current) => current.map((document) => {
          if (document.id === activeDocument.id) return nextDocument;
          if (nameAction === 'rename-project' && document.projectId === activeDocument.projectId) {
            return { ...document, projectName: name, state: { ...document.state, overview: { ...document.state.overview, projectName: name } } };
          }
          return document;
        }));
      }
      setNameAction(null);
    } catch (caught: any) {
      setNameError(caught?.message || 'Could not save the name.');
    } finally {
      setDocumentActionPending(false);
    }
  }

  async function deleteActiveDesign() {
    if (!activeDocument || documentActionPending) return;
    setError(null);
    setDocumentActionPending(true);
    try {
      await window.fraia.deleteDesign({
        projectDir: activeDocument.projectRootDir,
        projectId: activeDocument.projectId,
        designId: activeDocument.designId,
      });
      closeDocument(activeDocument.id);
    } catch (caught: any) {
      setError(caught?.message || 'Could not delete the design.');
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
      const nextState = normalizeWorkbenchState(await window.fraia.createProject({ projectDir, name: 'Untitled Project' }));
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

  async function saveActiveProject(saveAs: boolean, names?: FirstSaveNames) {
    if (!activeDocument || documentActionPending) return;
    if (activeDocument.managedUnsaved && !names) {
      setFirstSaveError(null);
      setFirstSaveOpen(true);
      return;
    }
    setError(null);
    setFirstSaveError(null);
    if (names) setFirstSaveOpen(false);
    setDocumentActionPending(true);
    try {
      const response = await window.fraia.saveProject({
        projectDir: activeDocument.projectRootDir,
        projectId: activeDocument.projectId,
        designId: activeDocument.designId,
        designIds: documents.filter((document) => document.projectId === activeDocument.projectId).map((document) => document.designId),
        projectName: names?.projectName,
        designName: names?.designName,
        suggestedName: names?.projectName ?? activeDocument.projectName,
        saveAs,
      });
      const designStates = response?.designStates ?? [response?.state ?? response];
      const nextDocuments = designStates.map(normalizeWorkbenchState).filter(Boolean).map(projectDocumentFromState);
      const nextState = normalizeWorkbenchState(response?.state ?? response);
      if (!nextState) return;
      setDocuments((current) => current.map((document) => nextDocuments.find((next: ProjectDocument) => next.id === document.id) ?? document));
      setActiveDocumentId(activeDocument.id);
    } catch (caught: any) {
      const message = caught?.message || 'Could not save the Fraia project.';
      if (names) {
        setFirstSaveError(message);
        setFirstSaveOpen(true);
      } else {
        setError(message);
      }
    } finally {
      setDocumentActionPending(false);
    }
  }

  useEffect(() => {
    const saveFromAppMenu = (event: Event) => {
      void saveActiveProject(Boolean((event as CustomEvent<{ saveAs?: boolean }>).detail?.saveAs));
    };
    window.addEventListener('fraia:save-project', saveFromAppMenu);
    const unsubscribeNativeMenu = window.fraia.onSaveProjectRequested?.((saveAs) => {
      void saveActiveProject(saveAs);
    });
    return () => {
      window.removeEventListener('fraia:save-project', saveFromAppMenu);
      unsubscribeNativeMenu?.();
    };
  }, [activeDocument, documentActionPending]);

  function closeDocument(documentId: string) {
    const closingIndex = documents.findIndex((document) => document.id === documentId);
    if (closingIndex < 0) return;
    void window.fraia.conversationCancelDesign?.({ designId: documents[closingIndex].designId });
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

  return (<>
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
        onNewBlankModel={() => { setNameError(null); setNameAction('create-design'); }}
        onRenameProject={() => { setNameError(null); setNameAction('rename-project'); }}
        onRenameDesign={() => { setNameError(null); setNameAction('rename-design'); }}
        onDeleteDesign={() => { void deleteActiveDesign(); }}
        documentActionPending={documentActionPending}
        documentError={error}
      />
    <FirstSaveDialog
      open={firstSaveOpen}
      projectName={activeDocument.projectName}
      designName={activeDocument.designName}
      pending={documentActionPending}
      error={firstSaveError}
      onOpenChange={setFirstSaveOpen}
      onContinue={(names) => { void saveActiveProject(false, names); }}
    />
    <NameDialog
      open={nameAction !== null}
      kind={nameAction ?? 'create-design'}
      initialValue={nameAction === 'rename-project' ? activeDocument.projectName : nameAction === 'rename-design' ? activeDocument.designName : ''}
      pending={documentActionPending}
      error={nameError}
      onOpenChange={(open) => { if (!open) setNameAction(null); }}
      onSubmit={(name) => { void submitNameAction(name); }}
    />
  </>);
}
