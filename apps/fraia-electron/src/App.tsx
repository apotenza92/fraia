import { useEffect, useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Spinner } from '@/components/ui/spinner';
import { AppShell } from './components/layout/AppShell';
import { loadDefaultProject, normalizeWorkbenchState } from './lib/defaultProject';
import {
  projectDocumentFromState,
  reorderProjectDocuments,
  upsertProjectDocument,
  type ProjectDocument,
} from './lib/projectDocuments';
import { useSystemTheme } from './lib/theme';
import type { WorkbenchState } from './lib/types';

export default function App() {
  const [documents, setDocuments] = useState<ProjectDocument[]>([]);
  const [activeDocumentId, setActiveDocumentId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [documentActionPending, setDocumentActionPending] = useState(false);
  useSystemTheme();
  const activeDocument = documents.find((document) => document.id === activeDocumentId) ?? documents[0] ?? null;

  useEffect(() => {
    let cancelled = false;
    loadDefaultProject()
      .then((loaded) => {
        if (cancelled || !loaded) return;
        const document = projectDocumentFromState(loaded);
        setDocuments([document]);
        setActiveDocumentId(document.id);
      })
      .catch((caught) => {
        if (!cancelled) setError(caught.message);
      });
    return () => {
      cancelled = true;
    };
  }, []);

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
      const projectDir = await window.fraia.pickDirectory();
      if (!projectDir) return;
      const existingDocument = documents.find((document) => document.projectDir === projectDir);
      if (existingDocument) {
        setActiveDocumentId(existingDocument.id);
        return;
      }
      const existingState = normalizeWorkbenchState(await window.fraia.refreshProjectIfExists(projectDir));
      if (existingState) {
        throw new Error('That folder already contains a Fraia model. Open its fraia.project.json file instead.');
      }
      const nextState = normalizeWorkbenchState(await window.fraia.createProject({ projectDir }));
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
    setDocuments((current) => current.length > 1
      ? current.filter((document) => document.id !== documentId)
      : current);
  }

  if (error && !activeDocument) {
    return (
      <main className="flex h-screen items-center justify-center p-6">
        <Alert variant="destructive" className="max-w-xl">
          <AlertTitle>Fraia backend unavailable</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </main>
    );
  }

  if (!activeDocument) {
    return (
      <main className="flex h-screen items-center justify-center gap-2">
        <Spinner />
        <span>Opening Fraia…</span>
      </main>
    );
  }

  const documentTabs = documents.map((document) => ({
    id: document.id,
    label: document.label,
    closable: documents.length > 1,
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
