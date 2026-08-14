import { useEffect, useMemo, useState } from 'react';
import { AlertCircle, Cuboid, FileText, FolderOpen, Library, Plus, Trash2 } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentGroup,
  AttachmentMedia,
  AttachmentTitle,
} from '@/components/ui/attachment';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { Field, FieldError, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/components/ui/sheet';
import { Spinner } from '@/components/ui/spinner';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { PdfPageBrowser, type PdfIndex } from './PdfPageBrowser';
import { DrawingInterpretationDialog } from './DrawingInterpretationDialog';
import { DxfSelectionDialog, type DxfIndexResult, type PreparedDxfSelection } from './DxfSelectionDialog';
import { IfcSelectionDialog, type IfcIndexResult } from './IfcSelectionDialog';
import { MeshSavedViewDialog, type MeshIndexResult } from './MeshSavedViewDialog';

type SourceRecord = {
  id: string;
  sha256: string;
  byte_size: number;
  detected_media_type: string;
  imported_at: string;
  aliases: Array<{ display_name: string; added_at: string; provenance: { origin_kind: string; supplied_name: string } }>;
  warnings?: Array<{ code: string; message: string }>;
};
type SourceDerivative = { id: string; kind: string; parser: string; parser_version: string; byte_size: number; created_at: string };
type ShelfItem = { id: string; label: string; kind: string; provenance?: { created_at?: string; created_by?: string; method?: string; derivative_id?: string }; source?: { source_id: string; source_sha256: string }; [key: string]: unknown };
type PdfPageIndex = { pageNumber: number; mediaBox: { x0: number; y0: number; x1: number; y1: number }; cropBox: { x0: number; y0: number; x1: number; y1: number }; rotationDegrees: number; userUnit: number; coordinateSpace: string; widthPoints: number; heightPoints: number; classification: string; extractionMethod: string; nativeTextCharacters: number; vectorPathOperations: number; embeddedImageCount: number };
type PdfInspection = { index: PdfIndex & { diagnostics: Array<{ code: string; message: string }> }; indexDerivative: SourceDerivative; resumed: boolean };
type FileInspection = { source: SourceRecord; derivatives: SourceDerivative[]; pdf?: PdfInspection; dxf?: DxfIndexResult; ifc?: IfcIndexResult; mesh?: MeshIndexResult; meshContent?: { sourceSha256: string; bytes: ArrayBuffer } };

function sourceName(source: SourceRecord) {
  return source.aliases[source.aliases.length - 1]?.display_name ?? source.id;
}

function visibleError(caught: unknown, fallback: string) {
  const message = caught instanceof Error ? caught.message : typeof caught === 'string' ? caught : '';
  return /\b(source|shelf|resource)\b/i.test(message) ? fallback : message || fallback;
}

function readableBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function ResourceLibrarySheet({
  open,
  initialView,
  projectDir,
  projectId,
  projectName,
  designId,
  designName,
  onOpenChange,
}: {
  open: boolean;
  initialView: 'sources' | 'shelf';
  projectDir: string;
  projectId: string;
  projectName: string;
  designId: string;
  designName: string;
  onOpenChange: (open: boolean) => void;
}) {
  const [view, setView] = useState(initialView);
  const [sources, setSources] = useState<SourceRecord[]>([]);
  const [shelfItems, setShelfItems] = useState<Record<string, ShelfItem>>({});
  const [pending, setPending] = useState(false);
  const [importState, setImportState] = useState<'uploading' | 'processing' | 'error' | 'done' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [inspection, setInspection] = useState<FileInspection | null>(null);
  const [pdfBrowserOpen, setPdfBrowserOpen] = useState(false);
  const [dxfBrowserOpen, setDxfBrowserOpen] = useState(false);
  const [ifcBrowserOpen, setIfcBrowserOpen] = useState(false);
  const [meshBrowserOpen, setMeshBrowserOpen] = useState(false);
  const [meshJobId, setMeshJobId] = useState<string | null>(null);
  const [dxfInterpretationParentId, setDxfInterpretationParentId] = useState<string | undefined>();
  const [renaming, setRenaming] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [interpretationReference, setInterpretationReference] = useState<ShelfItem | null>(null);
  const shelfSourceIds = useMemo(() => new Set(Object.values(shelfItems).map((item) => item.source?.source_id).filter(Boolean)), [shelfItems]);

  async function refresh() {
    const [sourceResult, shelfResult] = await Promise.all([
      window.fraia.listSources({ projectDir }),
      window.fraia.listShelf({ projectDir, designId }),
    ]);
    setSources(sourceResult.sources ?? []);
    setShelfItems((shelfResult.items ?? {}) as Record<string, ShelfItem>);
  }

  useEffect(() => { setView(initialView); }, [initialView, open]);
  useEffect(() => {
    if (!open) return;
    setError(null);
    void refresh().catch((caught) => setError(visibleError(caught, 'Could not load project files and design references.')));
  }, [open, projectDir, designId]);
  useEffect(() => window.fraia.onSourceImportProgress((progress) => {
    setImportState(progress.state);
    if (progress.state === 'error') setError(visibleError(progress.message, 'Could not import the file.'));
  }), []);
  useEffect(() => () => { if (meshJobId) void window.fraia.cancelMeshIndex({ jobId: meshJobId }); }, [meshJobId]);

  async function importSource() {
    setPending(true);
    setError(null);
    setImportState('uploading');
    try {
      const result = await window.fraia.importSource({ projectDir });
      if (!result) { setImportState(null); return; }
      await refresh();
      setImportState('done');
    } catch (caught: any) {
      setImportState('error');
      setError(visibleError(caught, 'Could not import the file.'));
    } finally {
      setPending(false);
    }
  }

  async function inspect(source: SourceRecord, openBrowser = false) {
    setError(null);
    try {
      const details = await window.fraia.inspectSource({ projectDir, sourceId: source.id });
      const pdf = source.detected_media_type === 'pdf'
        ? await window.fraia.indexPdfSource({ projectDir, sourceId: source.id })
        : undefined;
      const dxf = source.detected_media_type === 'dxf'
        ? await window.fraia.indexDxfSource({ projectDir, sourceId: source.id })
        : undefined;
      const ifc = source.detected_media_type === 'ifc_step'
        ? await window.fraia.indexIfcSource({ projectDir, sourceId: source.id })
        : undefined;
      let mesh: MeshIndexResult | undefined; let meshContent: { sourceSha256: string; bytes: ArrayBuffer } | undefined;
      if (['gltf', 'glb', 'obj', 'stl'].includes(source.detected_media_type)) {
        const started = await window.fraia.startMeshIndex({ projectDir, sourceId: source.id }); setMeshJobId(started.jobId);
        let status = started;
        while (status.status === 'running' || status.status === 'cancelling') { await new Promise((resolve) => setTimeout(resolve, 80)); status = await window.fraia.meshIndexStatus({ jobId: started.jobId }); }
        setMeshJobId(null);
        if (status.status !== 'completed' || !status.result) throw new Error(status.error || `3D indexing ${status.status}.`);
        const completedMesh = status.result as MeshIndexResult; mesh = completedMesh;
        const content = await window.fraia.readMeshContent({ projectDir, sourceId: source.id });
        if (content.sourceId !== completedMesh.index.source_id || content.sourceSha256 !== completedMesh.index.source_sha256 || content.byteSize !== source.byte_size) throw new Error('Managed 3D content identity did not match its index.');
        meshContent = { sourceSha256: content.sourceSha256, bytes: content.bytes };
      }
      setInspection({ ...details, pdf, dxf, ifc, mesh, meshContent } as FileInspection);
      if (openBrowser && pdf) setPdfBrowserOpen(true);
      if (openBrowser && dxf) {
        const interpretations = await window.fraia.listDrawingInterpretations({ projectDir, designId });
        setDxfInterpretationParentId(interpretations.headRevisionId);
        setDxfBrowserOpen(true);
      }
      if (openBrowser && ifc) {
        const interpretations = await window.fraia.listDrawingInterpretations({ projectDir, designId });
        setDxfInterpretationParentId(interpretations.headRevisionId);
        setIfcBrowserOpen(true);
      }
      if (openBrowser && mesh && meshContent) setMeshBrowserOpen(true);
    } catch (caught: any) {
      setError(visibleError(caught, 'Could not inspect the file.'));
    }
  }

  async function persistPreparedDxf(prepared: PreparedDxfSelection, format = 'DXF') {
    const expectedParentRevisionId = dxfInterpretationParentId;
    let referenceStored = false;
    try {
      const shelf = await window.fraia.upsertShelfItem({ projectDir, designId, item: prepared.shelf_item as ShelfItem });
      referenceStored = true;
      const interpretation = await window.fraia.createDrawingInterpretation({
        projectDir,
        designId,
        expectedParentRevisionId,
        authority: 'parser_adapter',
        revision: prepared.interpretation,
      });
      setShelfItems((shelf.items ?? {}) as Record<string, ShelfItem>);
      setDxfInterpretationParentId(interpretation.revisionId);
      const stored = (shelf.items?.[prepared.shelf_item.id] ?? prepared.shelf_item) as ShelfItem;
      setView('shelf');
      setInterpretationReference(stored);
    } catch (caught) {
      if (referenceStored) {
        try { await window.fraia.removeShelfItem({ projectDir, designId, itemId: prepared.shelf_item.id }); } catch { /* Keep the original persistence error. */ }
        await refresh().catch(() => undefined);
      }
      setError(visibleError(caught, `Could not add this ${format} content to the design references.`));
      throw caught;
    }
  }

  async function persistPreparedIfc(prepared: PreparedDxfSelection) {
    await persistPreparedDxf(prepared, 'IFC');
  }

  async function persistPreparedMesh(prepared: { shelf_item: ShelfItem }) {
    const shelf = await window.fraia.upsertShelfItem({ projectDir, designId, item: prepared.shelf_item });
    setShelfItems((shelf.items ?? {}) as Record<string, ShelfItem>); setView('shelf');
  }

  async function addShelfItem(item: ShelfItem) {
    try {
      const shelf = await window.fraia.upsertShelfItem({ projectDir, designId, item });
      setShelfItems((shelf.items ?? {}) as Record<string, ShelfItem>);
    } catch (caught: any) {
      setError(visibleError(caught, 'Could not add this PDF crop to the design references.'));
      throw caught;
    }
  }

  async function removeSource(source: SourceRecord) {
    setError(null);
    try {
      await window.fraia.removeSource({ projectDir, sourceId: source.id });
      await refresh();
    } catch (caught: any) {
      setError(visibleError(caught, 'Could not remove the file. Remove its design references first.'));
    }
  }

  async function removeShelfItem(item: ShelfItem) {
    setError(null);
    try {
      const shelf = await window.fraia.removeShelfItem({ projectDir, designId, itemId: item.id });
      setShelfItems((shelf.items ?? {}) as Record<string, ShelfItem>);
    } catch (caught: any) {
      setError(visibleError(caught, 'Could not remove the reference.'));
    }
  }

  async function saveShelfLabel(item: ShelfItem) {
    const label = renameValue.trim();
    if (!label) return;
    try {
      const shelf = await window.fraia.upsertShelfItem({ projectDir, designId, item: { ...item, label } });
      setShelfItems((shelf.items ?? {}) as Record<string, ShelfItem>);
      setRenaming(null);
    } catch (caught: any) {
      setError(visibleError(caught, 'Could not rename the reference.'));
    }
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="sm:max-w-xl" data-testid="resource-library-sheet">
        <SheetHeader>
          <SheetTitle>Files and references</SheetTitle>
          <SheetDescription>Import once, then choose what {designName} needs.</SheetDescription>
        </SheetHeader>
        <Tabs value={view} onValueChange={(value) => setView(value as 'sources' | 'shelf')} className="min-h-0 flex-1 px-4 pb-4">
          <TabsList className="w-full">
            <TabsTrigger value="sources">Project files</TabsTrigger>
            <TabsTrigger value="shelf">Design references</TabsTrigger>
          </TabsList>
          {error ? <Alert variant="destructive" className="mt-3"><AlertCircle /><AlertTitle>File or reference action failed</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
          {importState ? (
            <Attachment state={importState} className="mt-3 w-full" data-testid="source-import-status">
              <AttachmentMedia><FileText /></AttachmentMedia>
              <AttachmentContent><AttachmentTitle>Project file</AttachmentTitle><AttachmentDescription>{importState === 'uploading' ? 'Preparing the secure native file selection…' : importState === 'processing' ? 'Copying, validating, and indexing the file…' : importState === 'done' ? 'Import complete.' : 'Import failed.'}</AttachmentDescription></AttachmentContent>
              {importState === 'uploading' || importState === 'processing' ? <Spinner className="mr-3" /> : null}
            </Attachment>
          ) : null}
          <TabsContent value="sources" className="min-h-0">
            <div className="flex justify-end py-3"><Button disabled={pending} onClick={() => { void importSource(); }}><Plus data-icon="inline-start" />Add project file</Button></div>
            <ScrollArea className="h-[calc(100vh-15rem)]">
              {sources.length === 0 ? (
                <Empty><EmptyHeader><EmptyMedia variant="icon"><FolderOpen /></EmptyMedia><EmptyTitle>Start with a drawing or model</EmptyTitle><EmptyDescription>Add a PDF, image, CAD drawing, IFC model, or mesh. Every design can use it.</EmptyDescription></EmptyHeader><EmptyContent><Button onClick={() => { void importSource(); }}>Add project file</Button></EmptyContent></Empty>
              ) : (
                <div className="flex flex-col gap-3">
                <AttachmentGroup className="flex-col overflow-x-visible">
                  {sources.map((source) => (
                    <Attachment key={source.id} className="w-full" state="done">
                      <AttachmentMedia><FileText /></AttachmentMedia>
                      <AttachmentContent><AttachmentTitle>{sourceName(source)}</AttachmentTitle><AttachmentDescription>{source.detected_media_type.toUpperCase()} · {readableBytes(source.byte_size)} · imported {new Date(source.imported_at).toLocaleDateString()}</AttachmentDescription></AttachmentContent>
                      <Badge variant={shelfSourceIds.has(source.id) ? 'secondary' : 'outline'}>{shelfSourceIds.has(source.id) ? `In ${designName} references` : 'Project file'}</Badge>
                      <AttachmentActions>
                        <AttachmentAction size="sm" aria-label={`Inspect ${sourceName(source)}`} onClick={() => { void inspect(source); }}>Details</AttachmentAction>
                        <AttachmentAction size="sm" aria-label={source.detected_media_type === 'dxf' ? `Choose drawing content from ${sourceName(source)} for ${designName} references` : ['ifc_step','gltf','glb','obj','stl'].includes(source.detected_media_type) ? `Choose model content from ${sourceName(source)} for ${designName} references` : `Choose pages from ${sourceName(source)} for ${designName} references`} disabled={!['pdf', 'dxf', 'ifc_step', 'gltf', 'glb', 'obj', 'stl'].includes(source.detected_media_type)} onClick={() => { void inspect(source, true); }}>{source.detected_media_type === 'pdf' ? 'Choose pages' : 'Choose content'}</AttachmentAction>
                        <AttachmentAction aria-label={`Remove ${sourceName(source)} from project`} onClick={() => { void removeSource(source); }}><Trash2 /></AttachmentAction>
                      </AttachmentActions>
                    </Attachment>
                  ))}
                </AttachmentGroup>
                {inspection ? (
                  <div className="flex min-h-0 flex-col gap-2" data-testid="source-provenance">
                    <Alert><FileText /><AlertTitle>{sourceName(inspection.source)}</AlertTitle><AlertDescription>Ready to choose for {designName}.</AlertDescription></Alert>
                    {inspection.pdf ? <div className="flex flex-col gap-3">{inspection.pdf.index.pages.map((page) => <Item key={page.pageNumber} variant="outline"><ItemContent><ItemTitle>Page {page.pageNumber}</ItemTitle><ItemDescription>{page.classification.split('_').join(' ')} · {page.widthPoints} × {page.heightPoints} points · rotation {page.rotationDegrees}° · user unit {page.userUnit}</ItemDescription></ItemContent><ItemActions><Button size="sm" aria-label={`Open page ${page.pageNumber} of ${sourceName(inspection.source)}`} onClick={() => setPdfBrowserOpen(true)}>Open page</Button></ItemActions></Item>)}</div> : null}
                    {inspection.dxf ? <Alert><FileText /><AlertTitle>Indexed DXF drawing</AlertTitle><AlertDescription>{Object.keys(inspection.dxf.index.entities).length} entities across {[inspection.dxf.index.model_space_name, ...inspection.dxf.index.paper_layouts].length} layout{inspection.dxf.index.paper_layouts.length === 0 ? '' : 's'}. Units: {inspection.dxf.index.units ?? 'not declared; confirm scale before use'}.</AlertDescription></Alert> : null}
                    {inspection.ifc ? <Alert><FileText /><AlertTitle>Indexed IFC model</AlertTitle><AlertDescription>{Object.keys(inspection.ifc.index.objects).length} exact objects, {Object.keys(inspection.ifc.index.storeys).length} storeys, and {Object.keys(inspection.ifc.index.grids).length} grids. Fraia preserves IFC identities and transforms without authoring structural geometry.</AlertDescription></Alert> : null}
                    {inspection.mesh ? <Alert><Cuboid /><AlertTitle>Indexed 3D reference</AlertTitle><AlertDescription>{Object.keys(inspection.mesh.index.objects).length} exact objects · {inspection.mesh.index.vertex_count} vertices · {inspection.mesh.index.triangle_count} triangles. This remains read-only reference geometry.</AlertDescription></Alert> : null}
                    <Collapsible><CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>File provenance</CollapsibleTrigger><CollapsibleContent><Alert><FileText /><AlertTitle>Verified original</AlertTitle><AlertDescription>SHA-256 {inspection.source.sha256}. Supplied as {inspection.source.aliases[inspection.source.aliases.length - 1]?.provenance.supplied_name ?? 'unknown'}. {inspection.derivatives.length} derived artefact{inspection.derivatives.length === 1 ? '' : 's'} recorded.</AlertDescription></Alert></CollapsibleContent></Collapsible>
                  </div>
                ) : null}
                </div>
              )}
            </ScrollArea>
          </TabsContent>
          <TabsContent value="shelf" className="min-h-0">
            <p className="py-3 text-sm text-muted-foreground">Project files are shared. References are used only by this design.</p>
            <ScrollArea className="h-[calc(100vh-15rem)]">
              {Object.keys(shelfItems).length === 0 ? (
                <Empty><EmptyHeader><EmptyMedia variant="icon"><Library /></EmptyMedia><EmptyTitle>No design references yet</EmptyTitle><EmptyDescription>Choose the pages, drawing content, or model views that matter to {designName}.</EmptyDescription></EmptyHeader><EmptyContent><Button variant="outline" onClick={() => setView('sources')}>Choose from project files</Button></EmptyContent></Empty>
              ) : (
                <div className="flex flex-col gap-3">
                  {Object.values(shelfItems).map((item) => (
                    <Item key={item.id} variant="outline">
                      <ItemContent>
                        {renaming === item.id ? (
                          <Field data-invalid={!renameValue.trim()}><FieldLabel htmlFor={`shelf-label-${item.id}`}>Reference name</FieldLabel><Input id={`shelf-label-${item.id}`} value={renameValue} autoFocus aria-invalid={!renameValue.trim()} onChange={(event) => setRenameValue(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter') void saveShelfLabel(item); }} /><FieldError>{renameValue.trim() ? null : 'Enter a reference name.'}</FieldError></Field>
                        ) : <ItemTitle>{item.label}</ItemTitle>}
                        <ItemDescription>{item.kind.split('_').join(' ')}</ItemDescription>
                      </ItemContent>
                      <ItemActions>
                        <Button variant="outline" size="sm" aria-label={`Review interpretation for ${item.label}`} onClick={() => setInterpretationReference(item)}>Review interpretation</Button>
                        {renaming === item.id ? <Button size="sm" onClick={() => { void saveShelfLabel(item); }}>Save name</Button> : <Button variant="outline" size="sm" onClick={() => { setRenaming(item.id); setRenameValue(item.label); }}>Rename</Button>}
                        <Button variant="ghost" size="icon-sm" aria-label={`Remove ${item.label} from ${designName} references`} onClick={() => { void removeShelfItem(item); }}><Trash2 /></Button>
                      </ItemActions>
                    </Item>
                  ))}
                </div>
              )}
            </ScrollArea>
          </TabsContent>
        </Tabs>
        {inspection?.pdf ? <PdfPageBrowser open={pdfBrowserOpen} projectDir={projectDir} designName={designName} source={{ id: inspection.source.id, sha256: inspection.source.sha256, displayName: sourceName(inspection.source) }} index={inspection.pdf.index} indexDerivativeId={inspection.pdf.indexDerivative.id} onOpenChange={setPdfBrowserOpen} onAdd={(item) => addShelfItem(item as ShelfItem)} /> : null}
        {inspection?.dxf ? <DxfSelectionDialog open={dxfBrowserOpen} projectDir={projectDir} designId={designId} designName={designName} source={{ label: sourceName(inspection.source) }} indexed={inspection.dxf} interpretationParentRevisionId={dxfInterpretationParentId} onOpenChange={setDxfBrowserOpen} onPrepared={persistPreparedDxf} /> : null}
        {inspection?.ifc ? <IfcSelectionDialog open={ifcBrowserOpen} projectDir={projectDir} designId={designId} designName={designName} sourceLabel={sourceName(inspection.source)} indexed={inspection.ifc} interpretationParentRevisionId={dxfInterpretationParentId} onOpenChange={setIfcBrowserOpen} onPrepared={persistPreparedIfc} /> : null}
        {inspection?.mesh && inspection.meshContent ? <MeshSavedViewDialog open={meshBrowserOpen} projectDir={projectDir} designId={designId} designName={designName} sourceLabel={sourceName(inspection.source)} indexed={inspection.mesh} content={inspection.meshContent} onOpenChange={(next) => { setMeshBrowserOpen(next); if (!next && meshJobId) void window.fraia.cancelMeshIndex({ jobId: meshJobId }); }} onPrepared={persistPreparedMesh} /> : null}
        <DrawingInterpretationDialog open={interpretationReference !== null} projectDir={projectDir} projectId={projectId} designId={designId} designName={designName} reference={interpretationReference} onOpenChange={(nextOpen) => { if (!nextOpen) setInterpretationReference(null); }} />
      </SheetContent>
    </Sheet>
  );
}
