import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Check, Minus, MousePointer2, Plus, RotateCcw, Search } from 'lucide-react';
import type { PDFDocumentProxy, PDFPageProxy } from 'pdfjs-dist';
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Spinner } from '@/components/ui/spinner';
import { Textarea } from '@/components/ui/textarea';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';

type PdfBox = { x0: number; y0: number; x1: number; y1: number };
type PdfPageIndex = {
  pageNumber: number;
  mediaBox: PdfBox;
  cropBox: PdfBox;
  rotationDegrees: number;
  userUnit: number;
  coordinateSpace: string;
  widthPoints: number;
  heightPoints: number;
  classification: string;
  sourceToDisplayTransform: number[];
};
export type PdfIndex = {
  sourceId: string;
  sourceSha256: string;
  parser: string;
  parserVersion: string;
  pageCount: number;
  pages: PdfPageIndex[];
};
type Point = { x: number; y: number };
type DraftShape = { mode: 'rectangle' | 'polygon'; displayPoints: Point[]; sourcePoints: Point[] };
type PdfViewRoleInference = {
  suggestions: Array<{
    inferenceId: string;
    role: string;
    confidence: number;
    evidence: Array<{ text: string; pageNumber: number; sourceBox: { x0: number; y0: number; x1: number; y1: number }; evidenceKind: string }>;
    extractionMethod: string;
    materiallyConflicted: boolean;
    requiresQuestion: boolean;
  }>;
  diagnostics: Array<{ code: string; message: string }>;
};
type OcrCandidate = { candidateId: string; text: string; sourceBox: PdfBox; confidence: number };
const VIEW_ROLES = [{ value: 'plan', label: 'Plan' }, { value: 'elevation', label: 'Elevation' }, { value: 'section', label: 'Section' }, { value: 'detail', label: 'Detail' }, { value: 'schedule', label: 'Schedule' }, { value: 'reference', label: 'Reference' }];
const ORIENTATIONS = [{ value: 'north-up', label: 'North / vertical up' }, { value: 'east-up', label: 'East / horizontal up' }];
const DISTANCE_UNITS = [{ value: 'mm', label: 'mm' }, { value: 'm', label: 'm' }, { value: 'in', label: 'in' }, { value: 'ft', label: 'ft' }];

function ocrViewRole(candidates: OcrCandidate[], pageNumber: number): PdfViewRoleInference | null {
  const roles = [
    ['plan', /\bplan\b/i],
    ['elevation', /\belevation\b/i],
    ['section', /\bsection\b/i],
    ['detail', /\bdetail\b/i],
    ['schedule', /\bschedule\b/i],
  ] as const;
  const matches = roles.flatMap(([role, pattern]) => candidates
    .filter(({ text }) => pattern.test(text))
    .map((candidate) => ({ role, candidate })));
  if (matches.length === 0) return null;
  const best = [...matches].sort((left, right) => right.candidate.confidence - left.candidate.confidence)[0];
  const competing = matches.some((match) => match.role !== best.role && Math.abs(match.candidate.confidence - best.candidate.confidence) < 0.1);
  return {
    suggestions: [{
      inferenceId: best.candidate.candidateId,
      role: best.role,
      confidence: best.candidate.confidence,
      evidence: matches.filter(({ role }) => role === best.role).map(({ candidate }) => ({ text: candidate.text, pageNumber, sourceBox: candidate.sourceBox, evidenceKind: 'ocr_text_candidate' })),
      extractionMethod: 'ocr',
      materiallyConflicted: competing,
      requiresQuestion: competing || best.candidate.confidence < 0.75,
    }],
    diagnostics: [],
  };
}

export function displayPointToSource(point: Point, zoom: number, transform: number[]): Point {
  if (transform.length !== 6 || zoom <= 0) throw new Error('Invalid PDF source-to-display transform.');
  const [a, b, c, d, e, f] = transform;
  const determinant = a * d - b * c;
  if (Math.abs(determinant) < Number.EPSILON) throw new Error('PDF source-to-display transform is not invertible.');
  const displayX = point.x / zoom;
  const displayY = point.y / zoom;
  return {
    x: (d * (displayX - e) - c * (displayY - f)) / determinant,
    y: (-b * (displayX - e) + a * (displayY - f)) / determinant,
  };
}

export function cropRasterToSourceTransform(displayOrigin: Point, rasterScale: number, zoom: number, sourceToDisplay: number[]): [number, number, number, number, number, number] {
  const origin = displayPointToSource(displayOrigin, zoom, sourceToDisplay);
  const x = displayPointToSource({ x: displayOrigin.x + 1 / rasterScale, y: displayOrigin.y }, zoom, sourceToDisplay);
  const y = displayPointToSource({ x: displayOrigin.x, y: displayOrigin.y + 1 / rasterScale }, zoom, sourceToDisplay);
  return [x.x - origin.x, x.y - origin.y, y.x - origin.x, y.y - origin.y, origin.x, origin.y];
}

export function ocrRotationRadians(rotationDegrees: number): number {
  return -rotationDegrees * Math.PI / 180;
}

export function applyInferredDraftValues(current: { name: string; viewRole: string }, inferred: { name?: string; viewRole: string }, edited: { name: boolean; viewRole: boolean }) {
  return {
    name: edited.name || !inferred.name ? current.name : inferred.name,
    viewRole: edited.viewRole ? current.viewRole : inferred.viewRole,
  };
}

export function cachedInference(cache: Map<string, Promise<PdfViewRoleInference>>, key: string, load: () => Promise<PdfViewRoleInference>) {
  const existing = cache.get(key);
  if (existing) return existing;
  const pending = load();
  cache.set(key, pending);
  return pending;
}

function pageLabel(page: PdfPageIndex) {
  return `Page ${page.pageNumber}`;
}

function CanvasPage({ page, scale, onPage, onCanvas, className }: { page: PDFPageProxy; scale: number; onPage?: (page: PDFPageProxy) => void; onCanvas?: (canvas: HTMLCanvasElement | null) => void; className?: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    onPage?.(page);
    const canvas = canvasRef.current;
    if (!canvas) return;
    onCanvas?.(canvas);
    const viewport = page.getViewport({ scale });
    const ratio = window.devicePixelRatio || 1;
    canvas.width = Math.floor(viewport.width * ratio);
    canvas.height = Math.floor(viewport.height * ratio);
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    const context = canvas.getContext('2d');
    if (!context) return;
    const task = page.render({ canvasContext: context, canvas, viewport, transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0] });
    void task.promise.catch((error: unknown) => {
      if (error instanceof Error && error.name === 'RenderingCancelledException') return;
      queueMicrotask(() => { throw error; });
    });
    return () => { task.cancel(); onCanvas?.(null); };
  }, [onCanvas, onPage, page, scale]);
  return <canvas ref={canvasRef} className={className} aria-label={`Rendered PDF page ${page.pageNumber}`} />;
}

function Thumbnail({ document, pageIndex, selected, onSelect }: { document: PDFDocumentProxy; pageIndex: PdfPageIndex; selected: boolean; onSelect: () => void }) {
  const [page, setPage] = useState<PDFPageProxy | null>(null);
  useEffect(() => { void document.getPage(pageIndex.pageNumber).then(setPage); }, [document, pageIndex.pageNumber]);
  return (
    <Button variant={selected ? 'secondary' : 'outline'} className="h-auto w-full flex-col items-stretch p-2" onClick={onSelect} aria-label={`Open ${pageLabel(pageIndex)}`}>
      {page ? <CanvasPage page={page} scale={0.18} className="max-w-full self-center" /> : <Spinner />}
      <span>{pageLabel(pageIndex)}</span>
      <span className="text-muted-foreground">{pageIndex.classification.split('_').join(' ')}</span>
    </Button>
  );
}

export function PdfPageBrowser({ open, projectDir, designName, source, index, indexDerivativeId, onOpenChange, onAdd }: {
  open: boolean;
  projectDir: string;
  designName: string;
  source: { id: string; sha256: string; displayName: string };
  index: PdfIndex;
  indexDerivativeId: string;
  onOpenChange: (open: boolean) => void;
  onAdd: (item: Record<string, unknown>) => Promise<void>;
}) {
  const [document, setDocument] = useState<PDFDocumentProxy | null>(null);
  const [activePageNumber, setActivePageNumber] = useState(1);
  const [page, setPage] = useState<PDFPageProxy | null>(null);
  const [zoom, setZoom] = useState(1);
  const [query, setQuery] = useState('');
  const [mode, setMode] = useState<'pan' | 'rectangle' | 'polygon' | 'calibrate'>('rectangle');
  const [draft, setDraft] = useState<DraftShape | null>(null);
  const [drawing, setDrawing] = useState<Point[] | null>(null);
  const drawingRef = useRef<Point[] | null>(null);
  const [name, setName] = useState('');
  const [nameEdited, setNameEdited] = useState(false);
  const [notes, setNotes] = useState('');
  const [viewRole, setViewRole] = useState('plan');
  const [viewRoleEdited, setViewRoleEdited] = useState(false);
  const [orientation, setOrientation] = useState('north-up');
  const [knownDistance, setKnownDistance] = useState('');
  const [calibrationPoints, setCalibrationPoints] = useState<Point[]>([]);
  const panStart = useRef<{ x: number; y: number; left: number; top: number; viewport: HTMLElement } | null>(null);
  const pageCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const ocrInferenceCache = useRef(new Map<string, Promise<PdfViewRoleInference>>());
  const rememberPageCanvas = useCallback((canvas: HTMLCanvasElement | null) => { pageCanvasRef.current = canvas; }, []);
  const [unit, setUnit] = useState('m');
  const [confirmed, setConfirmed] = useState(false);
  const [inference, setInference] = useState<PdfViewRoleInference | null>(null);
  const [inferring, setInferring] = useState(false);
  const [inferenceStage, setInferenceStage] = useState<'native' | 'ocr'>('native');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const activeIndex = index.pages.find((candidate) => candidate.pageNumber === activePageNumber) ?? index.pages[0];
  const outdated = index.sourceId !== source.id || index.sourceSha256 !== source.sha256;
  const filteredPages = useMemo(() => index.pages.filter((candidate) => !query.trim() || String(candidate.pageNumber).includes(query.trim())), [index.pages, query]);
  const viewport = page?.getViewport({ scale: zoom }) ?? null;

  useEffect(() => {
    if (!open || outdated) return;
    let cancelled = false;
    void Promise.all([window.fraia.readPdfSource({ projectDir, sourceId: source.id }), import('pdfjs-dist')]).then(([bytes, pdfjs]) => {
      pdfjs.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;
      return pdfjs.getDocument({ data: bytes }).promise;
    }).then((next) => {
      if (cancelled) void next.cleanup(); else setDocument(next);
    }).catch((caught) => setError(caught?.message || 'Could not render this managed PDF.'));
    return () => { cancelled = true; };
  }, [open, outdated, projectDir, source.id]);
  useEffect(() => {
    if (!document) return;
    void document.getPage(activePageNumber).then((nextPage) => {
      const nextViewport = nextPage.getViewport({ scale: 1 });
      if (activeIndex && (Math.abs(nextViewport.width - activeIndex.widthPoints) > 0.02 || Math.abs(nextViewport.height - activeIndex.heightPoints) > 0.02)) {
        setError('The rendered PDF page dimensions do not match its persisted index. Re-index this project file before selecting areas.');
        return;
      }
      setPage(nextPage);
    }).catch((caught) => setError(caught?.message || 'Could not open this PDF page.'));
  }, [activePageNumber, document]);
  useEffect(() => () => { if (document) void document.cleanup(); }, [document]);
  useEffect(() => {
    if (!draft || !activeIndex || outdated) { setInference(null); return; }
    const xs = draft.sourcePoints.map((point) => point.x);
    const ys = draft.sourcePoints.map((point) => point.y);
    const inferenceKey = `${source.sha256}:${activePageNumber}:${draft.sourcePoints.map(({ x, y }) => `${x},${y}`).join(';')}`;
    let cancelled = false;
    let attemptedOcr = false;
    setInferring(true);
    setInferenceStage('native');
    void window.fraia.inferPdfViewRole({
      projectDir,
      sourceId: source.id,
      pageNumber: activePageNumber,
      crop: { x0: Math.min(...xs), y0: Math.min(...ys), x1: Math.max(...xs), y1: Math.max(...ys) },
      marginPoints: 36,
    }).then(async (nativeResult: PdfViewRoleInference) => {
      if (cancelled) return;
      let result = nativeResult;
      const canvas = pageCanvasRef.current;
      const nativeUnavailable = result.suggestions.length === 0
        && result.diagnostics.some(({ code }) => code === 'ocr_unavailable');
      if (nativeUnavailable && canvas) {
        attemptedOcr = true;
        setInferenceStage('ocr');
        const displayXs = draft.displayPoints.map(({ x }) => x);
        const displayYs = draft.displayPoints.map(({ y }) => y);
        const left = Math.max(0, Math.floor(Math.min(...displayXs) * (canvas.width / Number.parseFloat(canvas.style.width))));
        const top = Math.max(0, Math.floor(Math.min(...displayYs) * (canvas.height / Number.parseFloat(canvas.style.height))));
        const right = Math.min(canvas.width, Math.ceil(Math.max(...displayXs) * (canvas.width / Number.parseFloat(canvas.style.width))));
        const bottom = Math.min(canvas.height, Math.ceil(Math.max(...displayYs) * (canvas.height / Number.parseFloat(canvas.style.height))));
        if (right > left && bottom > top) {
          const cropCanvas = globalThis.document.createElement('canvas');
          cropCanvas.width = right - left;
          cropCanvas.height = bottom - top;
          cropCanvas.getContext('2d')?.drawImage(canvas, left, top, cropCanvas.width, cropCanvas.height, 0, 0, cropCanvas.width, cropCanvas.height);
          const bytes = new Uint8Array(await new Promise<ArrayBuffer>((resolve, reject) => cropCanvas.toBlob((blob) => blob ? blob.arrayBuffer().then(resolve, reject) : reject(new Error('Could not encode OCR crop.')), 'image/png')));
          const rasterScale = canvas.width / Number.parseFloat(canvas.style.width);
          const pending = cachedInference(ocrInferenceCache.current, inferenceKey, () => window.fraia.recognizePdfOcr({
              sourceId: source.id,
              sourceSha256: source.sha256,
              pageNumber: activePageNumber,
              rotationDegrees: activeIndex.rotationDegrees,
              sourceCoordinateSpace: activeIndex.coordinateSpace,
              crop: { x0: Math.min(...xs), y0: Math.min(...ys), x1: Math.max(...xs), y1: Math.max(...ys) },
              rasterWidth: cropCanvas.width,
              rasterHeight: cropCanvas.height,
              rasterToSourceTransform: cropRasterToSourceTransform({ x: Math.min(...displayXs), y: Math.min(...displayYs) }, rasterScale, zoom, activeIndex.sourceToDisplayTransform),
              ocrRotationRadians: ocrRotationRadians(activeIndex.rotationDegrees),
              nativeTextUsable: false,
              imageBytes: bytes,
            }).then((ocr) => ocr.status === 'completed'
              ? (ocrViewRole(ocr.candidates, activePageNumber) ?? { suggestions: [], diagnostics: [{ code: 'ocr_no_role', message: 'OCR found text, but no reliable view-role label in this crop.' }] })
              : { suggestions: [], diagnostics: ocr.diagnostics }));
          result = await pending;
        }
      }
      if (cancelled) return;
      setInference(result);
      const suggestion = result.suggestions[0];
      if (!suggestion) return;
      const citedText = suggestion.evidence[0]?.text.trim();
      if (citedText && !nameEdited) setName(citedText.slice(0, 120));
      if (!viewRoleEdited) setViewRole(suggestion.role);
    }).catch((caught) => {
      if (!cancelled) setInference({ suggestions: [], diagnostics: [{
        code: attemptedOcr ? 'ocr_failed' : 'view_role_inference_failed',
        message: attemptedOcr
          ? 'Fraia could not read this scanned drawing area. Choose the drawing view manually.'
          : (caught instanceof Error ? caught.message : 'Fraia could not infer a drawing view. Choose it manually.'),
      }] });
    }).finally(() => { if (!cancelled) setInferring(false); });
    return () => { cancelled = true; };
  }, [activeIndex, activePageNumber, draft, nameEdited, outdated, projectDir, source.id, source.sha256, viewRoleEdited, zoom]);

  function eventPoint(event: React.PointerEvent<SVGSVGElement>) {
    const bounds = event.currentTarget.getBoundingClientRect();
    return { x: (event.clientX - bounds.left) * ((viewport?.width ?? bounds.width) / bounds.width), y: (event.clientY - bounds.top) * ((viewport?.height ?? bounds.height) / bounds.height) };
  }
  function sourcePoint(point: Point) {
    return activeIndex ? displayPointToSource(point, zoom, activeIndex.sourceToDisplayTransform) : point;
  }
  function finishShape(points: Point[], shapeMode: 'rectangle' | 'polygon') {
    const displayPoints = shapeMode === 'rectangle' && points.length >= 2
      ? [points[0], { x: points[1].x, y: points[0].y }, points[1], { x: points[0].x, y: points[1].y }]
      : points;
    const sourcePoints = displayPoints.map(sourcePoint);
    setDraft({ mode: shapeMode, displayPoints, sourcePoints });
    setName(`${source.displayName} · page ${activePageNumber} ${shapeMode === 'rectangle' ? 'crop' : 'area'}`);
    setNameEdited(false);
    setViewRoleEdited(false);
    drawingRef.current = null;
    setDrawing(null);
  }
  function updateDrawing(points: Point[] | null) { drawingRef.current = points; setDrawing(points); }
  function pointerDown(event: React.PointerEvent<SVGSVGElement>) {
    if (mode === 'pan') {
      const viewportElement = event.currentTarget.closest('[data-slot="scroll-area"]')?.querySelector('[data-slot="scroll-area-viewport"]') as HTMLElement | null;
      if (viewportElement) {
        event.currentTarget.setPointerCapture(event.pointerId);
        panStart.current = { x: event.clientX, y: event.clientY, left: viewportElement.scrollLeft, top: viewportElement.scrollTop, viewport: viewportElement };
      }
      return;
    }
    const point = eventPoint(event);
    if (mode === 'calibrate') {
      const next = [...calibrationPoints, sourcePoint(point)].slice(-2);
      setCalibrationPoints(next);
      return;
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    if (mode === 'rectangle') updateDrawing([point, point]);
    else updateDrawing([...(drawingRef.current ?? []), point]);
  }
  function pointerMove(event: React.PointerEvent<SVGSVGElement>) {
    if (mode === 'pan' && panStart.current) {
      panStart.current.viewport.scrollLeft = panStart.current.left - (event.clientX - panStart.current.x);
      panStart.current.viewport.scrollTop = panStart.current.top - (event.clientY - panStart.current.y);
      return;
    }
    if (mode !== 'rectangle' || !drawingRef.current) return;
    updateDrawing([drawingRef.current[0], eventPoint(event)]);
  }
  function pointerUp(event: React.PointerEvent<SVGSVGElement>) {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    if (mode === 'pan') { panStart.current = null; return; }
    const current = drawingRef.current;
    if (mode === 'rectangle' && current && Math.abs(current[1].x - current[0].x) > 3 && Math.abs(current[1].y - current[0].y) > 3) finishShape(current, 'rectangle');
  }
  function resetDraft() { setDraft(null); updateDrawing(null); setName(''); setNameEdited(false); setViewRoleEdited(false); setNotes(''); setKnownDistance(''); setCalibrationPoints([]); setConfirmed(false); setInference(null); }
  async function addCrop() {
    if (!draft || !activeIndex || !name.trim()) return;
    const xs = draft.sourcePoints.map((point) => point.x);
    const ys = draft.sourcePoints.map((point) => point.y);
    const crop = { x: Math.min(...xs), y: Math.min(...ys), width: Math.max(...xs) - Math.min(...xs), height: Math.max(...ys) - Math.min(...ys), coordinate_space: activeIndex.coordinateSpace };
    const orientationVectors = orientation === 'east-up' ? { forward: [1, 0, 0], up: [0, 0, 1] } : { forward: [0, 1, 0], up: [0, 0, 1] };
    const item = {
      id: `source-${source.id}-page-${activePageNumber}-crop-${Date.now()}`,
      label: name.trim(),
      annotations: draft.mode === 'polygon' ? [{ id: 'selection-boundary', annotation_kind: 'polygon', points: draft.sourcePoints.map((point) => [point.x, point.y]), text: notes.trim() || undefined }] : [],
      confirmation: confirmed ? { confirmed: true, confirmed_by: 'user', confirmed_at: new Date().toISOString() } : { confirmed: false },
      provenance: { created_at: new Date().toISOString(), created_by: 'user', method: `pdf_${draft.mode}_crop`, derivative_id: indexDerivativeId },
      drawing_context: {
        view_role: viewRole,
        orientation: orientationVectors,
        ...(Number(knownDistance) > 0 && calibrationPoints.length === 2 ? { calibration: { first_point: [calibrationPoints[0].x, calibrationPoints[0].y], second_point: [calibrationPoints[1].x, calibrationPoints[1].y], known_distance: Number(knownDistance), unit, source_units_per_known_unit: Math.hypot(calibrationPoints[1].x - calibrationPoints[0].x, calibrationPoints[1].y - calibrationPoints[0].y) / Number(knownDistance), confirmed } } : {}),
      },
      kind: 'pdf_crop',
      source: { source_id: source.id, source_sha256: source.sha256 },
      page_number: activePageNumber,
      crop,
      layout: {
        media_box: { x: activeIndex.mediaBox.x0, y: activeIndex.mediaBox.y0, width: activeIndex.mediaBox.x1 - activeIndex.mediaBox.x0, height: activeIndex.mediaBox.y1 - activeIndex.mediaBox.y0, coordinate_space: activeIndex.coordinateSpace },
        crop_box: { x: activeIndex.cropBox.x0, y: activeIndex.cropBox.y0, width: activeIndex.cropBox.x1 - activeIndex.cropBox.x0, height: activeIndex.cropBox.y1 - activeIndex.cropBox.y0, coordinate_space: activeIndex.coordinateSpace },
        rotation_degrees: activeIndex.rotationDegrees,
        user_unit: activeIndex.userUnit,
      },
    };
    setSaving(true); setError(null);
    try { await onAdd(item); resetDraft(); } catch (caught: any) { setError(caught?.message || 'Could not add this crop to the design references.'); } finally { setSaving(false); }
  }

  const overlayPoints = drawing ?? draft?.displayPoints ?? [];
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="h-[92vh] w-[96vw]! max-w-[96vw]! p-0" data-testid="pdf-page-browser">
        <DialogHeader className="px-5 pt-5"><DialogTitle>Choose a drawing area</DialogTitle><DialogDescription>Draw around what {designName} needs. Fraia will suggest a name and drawing view.</DialogDescription></DialogHeader>
        {outdated ? <Alert variant="destructive" className="mx-5"><AlertTitle>Project file changed</AlertTitle><AlertDescription>This PDF index no longer matches the imported project file. Close this browser and re-index the current file.</AlertDescription></Alert> : null}
        {error ? <Alert variant="destructive" className="mx-5"><AlertTitle>PDF browser error</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
        <ResizablePanelGroup orientation="horizontal" className="min-h-0 flex-1">
          <ResizablePanel defaultSize="18%" minSize="14%">
            <div className="flex h-full flex-col gap-3 p-3">
              <Field><FieldLabel htmlFor="page-search">Find page</FieldLabel><div className="flex items-center gap-2"><Search /><Input id="page-search" inputMode="numeric" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Page number" /></div></Field>
              <ScrollArea className="min-h-0 flex-1"><div className="flex flex-col gap-2 pr-3">{document ? filteredPages.map((candidate) => <Thumbnail key={candidate.pageNumber} document={document} pageIndex={candidate} selected={candidate.pageNumber === activePageNumber} onSelect={() => { setActivePageNumber(candidate.pageNumber); resetDraft(); }} />) : <Spinner />}</div></ScrollArea>
            </div>
          </ResizablePanel>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize="57%" minSize="38%">
            <div className="flex h-full min-h-0 flex-col">
              <div className="flex flex-wrap items-center justify-between gap-2 p-3">
                <ToggleGroup value={[mode]} onValueChange={(value) => { const next = value[0] as typeof mode | undefined; if (next) setMode(next); }} variant="outline" size="sm">
                  <ToggleGroupItem value="pan" aria-label="Pan page"><MousePointer2 data-icon="inline-start" />Pan</ToggleGroupItem>
                  <ToggleGroupItem value="rectangle">Rectangle</ToggleGroupItem>
                  <ToggleGroupItem value="polygon">Polygon</ToggleGroupItem>
                  <ToggleGroupItem value="calibrate">Calibration points</ToggleGroupItem>
                </ToggleGroup>
                {mode === 'polygon' ? <Button variant="outline" size="sm" disabled={!drawing || drawing.length < 3} onClick={() => { if (drawing) finishShape(drawing, 'polygon'); }}>Finish polygon</Button> : null}
                <div className="flex items-center gap-2"><Button variant="outline" size="icon-sm" aria-label="Zoom out" onClick={() => setZoom((value) => Math.max(0.4, value - 0.2))}><Minus /></Button><Badge variant="outline">{Math.round(zoom * 100)}%</Badge><Button variant="outline" size="icon-sm" aria-label="Zoom in" onClick={() => setZoom((value) => Math.min(3, value + 0.2))}><Plus /></Button><Button variant="ghost" size="icon-sm" aria-label="Reset zoom" onClick={() => setZoom(1)}><RotateCcw /></Button></div>
              </div>
              <ScrollArea className="min-h-0 flex-1"><div className="flex min-h-full min-w-max items-center justify-center p-8">{page && viewport ? <div className="relative shadow-sm"><CanvasPage page={page} scale={zoom} onCanvas={rememberPageCanvas} /><svg viewBox={`0 0 ${viewport.width} ${viewport.height}`} className={`absolute inset-0 size-full ${mode === 'pan' ? 'cursor-grab' : 'cursor-crosshair'}`} aria-label="Drawing crop surface" role="application" tabIndex={0} onPointerDown={pointerDown} onPointerMove={pointerMove} onPointerUp={pointerUp} onPointerCancel={() => updateDrawing(null)} onKeyDown={(event) => { if (event.key === 'Enter' && mode === 'polygon' && drawing && drawing.length >= 3) finishShape(drawing, 'polygon'); if (event.key === 'Escape') { updateDrawing(null); setDraft(null); } }}><polygon points={overlayPoints.map((point) => `${point.x},${point.y}`).join(' ')} fill="color-mix(in srgb, var(--primary) 18%, transparent)" stroke="var(--primary)" strokeWidth={2 / zoom} />{calibrationPoints.map((point, index) => { const [a, b, c, d, e, f] = activeIndex.sourceToDisplayTransform; const x = (a * point.x + c * point.y + e) * zoom; const y = (b * point.x + d * point.y + f) * zoom; return <circle key={index} cx={x} cy={y} r={5 / zoom} fill="var(--destructive)" />; })}</svg></div> : <Spinner />}</div></ScrollArea>
            </div>
          </ResizablePanel>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize="25%" minSize="20%">
            <ScrollArea className="h-full"><div className="p-4"><FieldGroup>
              <Field><FieldLabel htmlFor="crop-name">Name</FieldLabel><Input id="crop-name" value={name} onChange={(event) => { setName(event.target.value); setNameEdited(true); }} placeholder="Select an area first" disabled={!draft} /><FieldDescription>Fraia fills this from drawing evidence when it can. You can change it.</FieldDescription></Field>
              {inferring ? <Alert data-testid="pdf-inference-progress"><Spinner /><AlertTitle>{inferenceStage === 'ocr' ? 'Reading scanned text…' : 'Checking native PDF evidence'}</AlertTitle><AlertDescription>{inferenceStage === 'ocr' ? 'Fraia is reading only this selected drawing area. The result remains unconfirmed until you review it.' : 'Fraia is ranking view roles from text inside and near this exact crop.'}</AlertDescription></Alert> : null}
              {inference?.suggestions[0] ? <Alert data-testid="pdf-view-role-inference"><Check /><AlertTitle className="flex items-center gap-2"><Badge variant="secondary">Fraia inferred</Badge>{inference.suggestions[0].role} · {Math.round(inference.suggestions[0].confidence * 100)}%</AlertTitle><AlertDescription className="flex flex-col gap-1"><span>{inference.suggestions[0].evidence.map((item) => `Page ${item.pageNumber}: “${item.text}”`).join(' · ')}</span><span>The reference name uses cited crop text. Fraia did not read a title-block or drawing register field.</span></AlertDescription></Alert> : null}
              {inference?.suggestions[0]?.requiresQuestion || inference?.suggestions[0]?.materiallyConflicted ? <Alert variant="destructive"><AlertTitle>Which drawing view is this?</AlertTitle><AlertDescription>Native text evidence is materially ambiguous. Choose the correct view role before Fraia treats it as more than an explicit proposal assumption.</AlertDescription></Alert> : null}
              {!inferring && inference && inference.suggestions.length === 0 ? <Alert><AlertTitle>Choose the drawing view manually</AlertTitle><AlertDescription>{inference.diagnostics.map((item) => item.message).join(' ') || 'No reliable native or OCR text evidence was found.'}</AlertDescription></Alert> : null}
              <Field><FieldLabel>View role</FieldLabel><Select items={VIEW_ROLES} value={viewRole} onValueChange={(value) => { setViewRole(value ?? 'reference'); setViewRoleEdited(true); }}><SelectTrigger className="w-full"><SelectValue /></SelectTrigger><SelectContent><SelectGroup>{VIEW_ROLES.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectGroup></SelectContent></Select></Field>
              <Collapsible><CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>More options</CollapsibleTrigger><CollapsibleContent><FieldGroup>
                <Field><FieldLabel htmlFor="crop-notes">Notes</FieldLabel><Textarea id="crop-notes" value={notes} onChange={(event) => setNotes(event.target.value)} disabled={!draft} placeholder="Anything Fraia should know about this area?" /></Field>
                <Field><FieldLabel>Orientation</FieldLabel><Select items={ORIENTATIONS} value={orientation} onValueChange={(value) => setOrientation(value ?? 'north-up')}><SelectTrigger className="w-full"><SelectValue /></SelectTrigger><SelectContent><SelectGroup>{ORIENTATIONS.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectGroup></SelectContent></Select></Field>
                <Field><FieldLabel htmlFor="known-distance">Scale from two points</FieldLabel><div className="flex gap-2"><Input id="known-distance" type="number" min="0" value={knownDistance} onChange={(event) => setKnownDistance(event.target.value)} disabled={!draft || calibrationPoints.length !== 2} /><Select items={DISTANCE_UNITS} value={unit} onValueChange={(value) => setUnit(value ?? 'm')}><SelectTrigger><SelectValue /></SelectTrigger><SelectContent><SelectGroup>{DISTANCE_UNITS.map((item) => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectGroup></SelectContent></Select></div><FieldDescription>Optional. Choose Calibration points, then select two known points.</FieldDescription></Field>
                <Alert><Check /><AlertTitle>Exact source</AlertTitle><AlertDescription>{draft ? `${draft.sourcePoints.length} points on ${pageLabel(activeIndex)} in ${activeIndex.coordinateSpace}. File ${source.sha256.slice(0, 12)}…` : 'Select an area to see its exact source.'}</AlertDescription></Alert>
              </FieldGroup></CollapsibleContent></Collapsible>
              <Field orientation="horizontal"><Checkbox id="confirm-crop" checked={confirmed} onCheckedChange={(value) => setConfirmed(value === true)} disabled={!draft} /><FieldLabel htmlFor="confirm-crop">I checked this crop, role, orientation, and any calibration.</FieldLabel></Field>
              {!draft ? <Alert><Check /><AlertTitle>Select an area</AlertTitle><AlertDescription>Draw a rectangle, or choose Polygon and press Enter when finished.</AlertDescription></Alert> : null}
            </FieldGroup></div></ScrollArea>
          </ResizablePanel>
        </ResizablePanelGroup>
        <DialogFooter className="px-5 pb-5"><Button variant="outline" onClick={() => onOpenChange(false)}>Close</Button><Button disabled={!draft || !name.trim() || saving || outdated} onClick={() => { void addCrop(); }}>{saving ? <Spinner data-icon="inline-start" /> : <Check data-icon="inline-start" />}Add design reference</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
