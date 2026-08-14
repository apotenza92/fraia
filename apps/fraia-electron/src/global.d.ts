import type {
  ConversationCreateRequest,
  ConversationAnalysisRequest,
  ConversationComparisonRequest,
  ConversationComparisonResponse,
  ConversationEvidenceResponse,
  ConversationForkRequest,
  ConversationMessageRequest,
  ConversationFactsUpdateRequest,
  ConversationProposalActionRequest,
  ConversationProposalRequest,
  ConversationTransportState,
  ConversationWorkingCopyCommitRequest,
  ConversationWorkingCopyOperationRequest,
  ConversationWorkingCopyOpenRequest,
} from '@/lib/conversationWorkspace';

type AnalysisAttemptResponse = {
  attemptId: string;
  projectId: string;
  revisionId: string;
  authoredSnapshotId: string;
  evidenceId: string;
  stage: 'preparing' | 'resolving' | 'solving' | 'collecting';
  status: 'running' | 'cancelling' | 'completed' | 'failed' | 'unsupported' | 'cancelled';
  elapsedMillis: number;
  canonicalRunId?: string;
  diagnostics?: string[];
};
import type {
  DesignRunList,
  DesignRunStatusProjection,
  DrawingInterpretation,
  DrawingInterpretationList,
  InspectedDesignRun,
} from '@/lib/engineeringEvidence';

type SourceRecord = {
  id: string;
  sha256: string;
  byte_size: number;
  detected_media_type: string;
  media_type: string;
  imported_at: string;
  aliases: Array<{ display_name: string; added_at: string; provenance: { origin_kind: string; supplied_name: string } }>;
  units?: string;
  coordinate_system?: string;
  warnings?: Array<{ code: string; message: string }>;
};
type SourceDerivative = { id: string; kind: string; parser: string; parser_version: string; byte_size: number; media_type: string; created_at: string };
type SourceImportResponse = { record: SourceRecord; job: { id: string; status: string; alias: string; started_at: string; completed_at: string }; deduplicated: boolean };
type PdfPageIndex = { pageNumber: number; mediaBox: { x0: number; y0: number; x1: number; y1: number }; cropBox: { x0: number; y0: number; x1: number; y1: number }; rotationDegrees: number; userUnit: number; coordinateSpace: string; widthPoints: number; heightPoints: number; nativeTextCharacters: number; vectorPathOperations: number; embeddedImageCount: number; classification: string; extractionMethod: string; sourceToDisplayTransform: number[] };
type PdfIndexResponse = { index: { sourceId: string; sourceSha256: string; parser: string; parserVersion: string; pageCount: number; pages: PdfPageIndex[]; diagnostics: Array<{ code: string; message: string }> }; indexDerivative: SourceDerivative; resumed: boolean };
type ShelfItem = { id: string; label: string; confirmation?: { confirmed?: boolean }; [key: string]: unknown };
type ShelfDocument = { schema_version: string; design_id: string; items: Record<string, ShelfItem> };

export {};
declare global {
  interface Window {
    fraia: Record<string, (...args: any[]) => Promise<any>> & {
      applicationMetadata?: () => Promise<{
        channel: 'stable' | 'beta';
        productName: string;
        userDataDirectoryName: string;
      }>;
      pickProjectFile: () => Promise<string | null>;
      createUntitledProject: () => Promise<string>;
      saveProject: (request: {
        projectDir: string;
        projectId: string;
        designId: string;
        designIds?: string[];
        projectName?: string;
        designName?: string;
        suggestedName: string;
        saveAs: boolean;
      }) => Promise<WorkbenchState | null>;
      createDesign: (request: { projectDir: string; projectId: string; designName: string }) => Promise<WorkbenchState>;
      activateDesign: (request: { projectDir: string; projectId: string; designId: string }) => Promise<WorkbenchState>;
      renameDesign: (request: { projectDir: string; projectId: string; projectName: string; designId: string; designName: string }) => Promise<WorkbenchState>;
      deleteDesign: (request: { projectDir: string; projectId: string; designId: string }) => Promise<unknown>;
      importSource: (request: { projectDir: string }) => Promise<SourceImportResponse | null>;
      onSourceImportProgress: (listener: (progress: { state: 'uploading' | 'processing' | 'done' | 'error'; message?: string }) => void) => () => void;
      listSources: (request: { projectDir: string }) => Promise<{ sources: SourceRecord[] }>;
      inspectSource: (request: { projectDir: string; sourceId: string }) => Promise<{ source: SourceRecord; derivatives: SourceDerivative[] }>;
      indexPdfSource: (request: { projectDir: string; sourceId: string }) => Promise<PdfIndexResponse>;
      indexDxfSource: (request: { projectDir: string; sourceId: string }) => Promise<any>;
      prepareDxfSelection: (request: Record<string, unknown>) => Promise<{ shelf_item: ShelfItem; interpretation: Record<string, unknown> }>;
      inferPdfViewRole: (request: { projectDir: string; sourceId: string; pageNumber: number; crop: { x0: number; y0: number; x1: number; y1: number }; marginPoints: number }) => Promise<any>;
      recognizePdfOcr: (request: { sourceId: string; sourceSha256: string; pageNumber: number; rotationDegrees: number; sourceCoordinateSpace: string; crop: { x0: number; y0: number; x1: number; y1: number }; rasterWidth: number; rasterHeight: number; rasterToSourceTransform: [number, number, number, number, number, number]; ocrRotationRadians?: number; nativeTextUsable: false; imageBytes: Uint8Array }) => Promise<{ schema: string; status: 'completed' | 'failed' | 'timed_out' | 'cancelled' | 'unavailable'; sourceId: string; sourceSha256: string; pageNumber: number; confirmation: 'unconfirmed'; requiresConfirmation: true; candidates: Array<{ candidateId: string; text: string; sourceBox: { x0: number; y0: number; x1: number; y1: number }; rasterBox: { x0: number; y0: number; x1: number; y1: number }; confidence: number; confirmation: 'unconfirmed'; requiresConfirmation: true }>; diagnostics: Array<{ code: string; message: string }> }>;
      indexIfcSource: (request: { projectDir: string; sourceId: string }) => Promise<any>;
      prepareIfcSelection: (request: Record<string, unknown>) => Promise<{ shelf_item: ShelfItem; interpretation: Record<string, unknown> }>;
      startMeshIndex: (request: { projectDir: string; sourceId: string }) => Promise<any>;
      meshIndexStatus: (request: { jobId: string }) => Promise<any>;
      cancelMeshIndex: (request: { jobId: string }) => Promise<any>;
      readMeshContent: (request: { projectDir: string; sourceId: string }) => Promise<{ sourceId: string; sourceSha256: string; mediaType: string; byteSize: number; bytes: ArrayBuffer }>;
      prepareMeshSavedView: (request: Record<string, unknown>) => Promise<any>;
      readPdfSource: (request: { projectDir: string; sourceId: string }) => Promise<Uint8Array>;
      removeSource: (request: { projectDir: string; sourceId: string }) => Promise<unknown>;
      listShelf: (request: { projectDir: string; designId: string }) => Promise<ShelfDocument>;
      upsertShelfItem: (request: { projectDir: string; designId: string; item: ShelfItem }) => Promise<ShelfDocument>;
      removeShelfItem: (request: { projectDir: string; designId: string; itemId: string }) => Promise<ShelfDocument>;
      listDrawingInterpretations: (request: { projectDir: string; designId: string }) => Promise<DrawingInterpretationList>;
      inspectDrawingInterpretation: (request: { projectDir: string; designId: string; revisionId: string }) => Promise<DrawingInterpretation>;
      createDrawingInterpretation: (request: Record<string, unknown>) => Promise<DrawingInterpretation>;
      confirmDrawingObservations: (request: Record<string, unknown>) => Promise<DrawingInterpretation>;
      correctDrawingObservation: (request: Record<string, unknown>) => Promise<DrawingInterpretation>;
      reconcileDrawingInterpretation: (request: Record<string, unknown>) => Promise<DrawingInterpretation>;
      resolveDrawingInterpretationConflict: (request: Record<string, unknown>) => Promise<DrawingInterpretation>;
      listDesignRuns: (request: { projectDir: string; designId: string }) => Promise<DesignRunList>;
      inspectDesignRun: (request: { projectDir: string; designId: string; runId: string }) => Promise<InspectedDesignRun>;
      listDesignRunStatuses: (request: { projectDir: string; designId: string; inspectedSnapshotId: string; ancestorSnapshotIds: string[] }) => Promise<DesignRunStatusProjection[]>;
      onSaveProjectRequested?: (listener: (saveAs: boolean) => void) => () => void;
      conversationCreate?: (request: ConversationCreateRequest) => Promise<ConversationTransportState>;
      conversationConverse?: (request: ConversationMessageRequest) => Promise<ConversationTransportState>;
      conversationFacts?: (request: ConversationFactsUpdateRequest) => Promise<ConversationTransportState>;
      conversationAgentRespond?: (request: {
        projectDir: string;
        packageProjectId: string;
        projectId: string;
        designId: string;
        conversationId: string;
        expectedHeadRevisionId: string;
        expectedSnapshotId: string;
        text: string;
        shelfItemIds: string[];
        drawingInterpretationRevisionIds: string[];
        turnId: string;
      }) => Promise<import('@/lib/conversationWorkspace').ConversationAgentRespondResponse>;
      conversationCancelDesign?: (request: { designId: string }) => Promise<{ cancelled: number }>;
      conversationAnalyse?: (request: ConversationAnalysisRequest) => Promise<ConversationEvidenceResponse>;
      startAnalysisAttempt: (request: Record<string, unknown>) => Promise<AnalysisAttemptResponse>;
      analysisAttemptStatus: (request: { projectId: string; attemptId: string }) => Promise<AnalysisAttemptResponse>;
      cancelAnalysisAttempt: (request: { projectId: string; attemptId: string }) => Promise<AnalysisAttemptResponse>;
      conversationCompare?: (request: ConversationComparisonRequest) => Promise<ConversationComparisonResponse>;
      conversationPropose?: (request: ConversationProposalRequest) => Promise<unknown>;
      conversationAccept?: (request: ConversationProposalActionRequest) => Promise<{
        revisionId: string;
        snapshotId: string;
        parentRevisionId: string | null;
        author: 'agent' | 'manual' | 'system' | 'user';
        agentProvenance?: { provider: string; model: string; turnId: string } | null;
      }>;
      conversationReject?: (request: ConversationProposalActionRequest) => Promise<unknown>;
      conversationFork?: (request: ConversationForkRequest) => Promise<ConversationTransportState>;
      conversationWorkingCopyOpen?: (request: ConversationWorkingCopyOpenRequest) => Promise<{
        workingCopyId: string;
        sourceRevisionId: string;
        sourceSnapshotId: string;
      }>;
      conversationWorkingCopyApply?: (request: ConversationWorkingCopyOperationRequest) => Promise<unknown>;
      conversationWorkingCopyCommit?: (request: ConversationWorkingCopyCommitRequest) => Promise<{
        revisionId: string;
        snapshotId: string;
        parentRevisionId: string | null;
        author: 'agent' | 'manual' | 'system' | 'user';
        agentProvenance?: { provider: string; model: string; turnId: string } | null;
      }>;
      updateStatus?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      checkForUpdates?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      setUpdateFrequency?: (frequency: import('@/lib/updateStatus').UpdateFrequency) => Promise<import('@/lib/updateStatus').UpdateStatus>;
      installUpdate?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      onUpdateStatus?: (listener: (status: import('@/lib/updateStatus').UpdateStatus) => void) => () => void;
      onOpenUpdateDialog?: (listener: () => void) => () => void;
      defaultProjectDir: () => Promise<string>;
      setThemeSource?: (themeSource: 'light' | 'dark' | 'system') => Promise<{ ok: boolean; themeSource?: string }>;
      reloadWindow?: () => Promise<{ ok: boolean }>;
      forceReloadWindow?: () => Promise<{ ok: boolean }>;
      quitApp?: () => Promise<{ ok: boolean }>;
    };
  }
}
