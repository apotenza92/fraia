const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('fraia', {
  health: () => ipcRenderer.invoke('fraia:health'),
  applicationMetadata: () => ipcRenderer.invoke('fraia:applicationMetadata'),
  defaultProjectDir: () => ipcRenderer.invoke('fraia:defaultProjectDir'),
  createUntitledProject: () => ipcRenderer.invoke('fraia:createUntitledProject'),
  saveProject: (payload) => ipcRenderer.invoke('fraia:saveProject', {
    projectDir: payload.projectDir,
    projectId: payload.projectId,
    designId: payload.designId,
    designIds: payload.designIds,
    projectName: payload.projectName,
    designName: payload.designName,
    suggestedName: payload.suggestedName,
    saveAs: payload.saveAs === true,
  }),
  onSaveProjectRequested: (listener) => {
    const handler = (_event, saveAs) => listener(saveAs);
    ipcRenderer.on('fraia:saveProjectRequested', handler);
    return () => ipcRenderer.removeListener('fraia:saveProjectRequested', handler);
  },
  pickProjectFile: () => ipcRenderer.invoke('fraia:pickProjectFile'),
  createProject: (payload) => ipcRenderer.invoke('fraia:createProject', payload),
  openProject: (payload) => ipcRenderer.invoke('fraia:openProject', payload),
  importSource: (payload) => ipcRenderer.invoke('fraia:importSource', { projectDir: payload.projectDir }),
  onSourceImportProgress: (listener) => {
    const handler = (_event, progress) => listener(progress);
    ipcRenderer.on('fraia:sourceImportProgress', handler);
    return () => ipcRenderer.removeListener('fraia:sourceImportProgress', handler);
  },
  listSources: (payload) => ipcRenderer.invoke('fraia:listSources', payload),
  inspectSource: (payload) => ipcRenderer.invoke('fraia:inspectSource', payload),
  indexPdfSource: (payload) => ipcRenderer.invoke('fraia:indexPdfSource', payload),
  indexDxfSource: (payload) => ipcRenderer.invoke('fraia:indexDxfSource', payload),
  prepareDxfSelection: (payload) => ipcRenderer.invoke('fraia:prepareDxfSelection', payload),
  inferPdfViewRole: (payload) => ipcRenderer.invoke('fraia:inferPdfViewRole', payload),
  recognizePdfOcr: (payload) => ipcRenderer.invoke('fraia:recognizePdfOcr', payload),
  indexIfcSource: (payload) => ipcRenderer.invoke('fraia:indexIfcSource', payload),
  prepareIfcSelection: (payload) => ipcRenderer.invoke('fraia:prepareIfcSelection', payload),
  startMeshIndex: (payload) => ipcRenderer.invoke('fraia:startMeshIndex', payload),
  meshIndexStatus: (payload) => ipcRenderer.invoke('fraia:meshIndexStatus', payload),
  cancelMeshIndex: (payload) => ipcRenderer.invoke('fraia:cancelMeshIndex', payload),
  readMeshContent: (payload) => ipcRenderer.invoke('fraia:readMeshContent', payload),
  prepareMeshSavedView: (payload) => ipcRenderer.invoke('fraia:prepareMeshSavedView', payload),
  readPdfSource: (payload) => ipcRenderer.invoke('fraia:readPdfSource', { projectDir: payload.projectDir, sourceId: payload.sourceId }),
  removeSource: (payload) => ipcRenderer.invoke('fraia:removeSource', payload),
  listShelf: (payload) => ipcRenderer.invoke('fraia:listShelf', payload),
  upsertShelfItem: (payload) => ipcRenderer.invoke('fraia:upsertShelfItem', payload),
  removeShelfItem: (payload) => ipcRenderer.invoke('fraia:removeShelfItem', payload),
  listDrawingInterpretations: (payload) => ipcRenderer.invoke('fraia:listDrawingInterpretations', payload),
  inspectDrawingInterpretation: (payload) => ipcRenderer.invoke('fraia:inspectDrawingInterpretation', payload),
  createDrawingInterpretation: (payload) => ipcRenderer.invoke('fraia:createDrawingInterpretation', payload),
  confirmDrawingObservations: (payload) => ipcRenderer.invoke('fraia:confirmDrawingObservations', payload),
  correctDrawingObservation: (payload) => ipcRenderer.invoke('fraia:correctDrawingObservation', payload),
  reconcileDrawingInterpretation: (payload) => ipcRenderer.invoke('fraia:reconcileDrawingInterpretation', payload),
  resolveDrawingInterpretationConflict: (payload) => ipcRenderer.invoke('fraia:resolveDrawingInterpretationConflict', payload),
  listDesignRuns: (payload) => ipcRenderer.invoke('fraia:listDesignRuns', payload),
  inspectDesignRun: (payload) => ipcRenderer.invoke('fraia:inspectDesignRun', payload),
  listDesignRunStatuses: (payload) => ipcRenderer.invoke('fraia:listDesignRunStatuses', payload),
  createDesign: (payload) => ipcRenderer.invoke('fraia:createDesign', payload),
  activateDesign: (payload) => ipcRenderer.invoke('fraia:activateDesign', payload),
  renameDesign: (payload) => ipcRenderer.invoke('fraia:renameDesign', payload),
  deleteDesign: (payload) => ipcRenderer.invoke('fraia:deleteDesign', payload),
  savePlanningDraft: (payload) => ipcRenderer.invoke('fraia:savePlanningDraft', payload),
  updateDesignOptionDecision: (payload) => ipcRenderer.invoke('fraia:updateDesignOptionDecision', payload),
  rawDesignOptionAnalysis: (payload) => ipcRenderer.invoke('fraia:rawDesignOptionAnalysis', payload),
  prepareSchemaHandoff: (payload) => ipcRenderer.invoke('fraia:prepareSchemaHandoff', payload),
  reviewReply: (payload) => ipcRenderer.invoke('fraia:reviewReply', payload),
  preSolveCoordinator: (payload) => ipcRenderer.invoke('fraia:preSolveCoordinator', payload),
  generateDesignOptions: (payload) => ipcRenderer.invoke('fraia:generateDesignOptions', payload),
  resetBaseModelGuide: (payload) => ipcRenderer.invoke('fraia:resetBaseModelGuide', payload),
  agentStartSession: (payload) => ipcRenderer.invoke('fraia:agentStartSession', payload),
  agentRespondSession: (payload) => ipcRenderer.invoke('fraia:agentRespondSession', payload),
  agentCancelSession: (payload) => ipcRenderer.invoke('fraia:agentCancelSession', payload),
  agentProviderStatus: (payload) => ipcRenderer.invoke('fraia:agentProviderStatus', payload),
  aiProviders: () => ipcRenderer.invoke('fraia:aiProviders'),
  aiRefreshCatalog: () => ipcRenderer.invoke('fraia:aiRefreshCatalog'),
  aiStartOAuth: (payload) => ipcRenderer.invoke('fraia:aiStartOAuth', payload),
  aiCancelAuth: (payload) => ipcRenderer.invoke('fraia:aiCancelAuth', payload),
  aiDisconnect: (payload) => ipcRenderer.invoke('fraia:aiDisconnect', payload),
  onAiRuntimeStatus: (listener) => {
    const handler = (_event, status) => listener(status);
    ipcRenderer.on('fraia:aiRuntimeStatus', handler);
    return () => ipcRenderer.removeListener('fraia:aiRuntimeStatus', handler);
  },
  updateStatus: () => ipcRenderer.invoke('fraia:updateStatus'),
  checkForUpdates: () => ipcRenderer.invoke('fraia:checkForUpdates'),
  setUpdateFrequency: (frequency) => ipcRenderer.invoke('fraia:setUpdateFrequency', frequency),
  installUpdate: () => ipcRenderer.invoke('fraia:installUpdate'),
  onUpdateStatus: (listener) => {
    const handler = (_event, status) => listener(status);
    ipcRenderer.on('fraia:updateStatus', handler);
    return () => ipcRenderer.removeListener('fraia:updateStatus', handler);
  },
  onOpenUpdateDialog: (listener) => {
    const handler = () => listener();
    ipcRenderer.on('fraia:openUpdateDialog', handler);
    return () => ipcRenderer.removeListener('fraia:openUpdateDialog', handler);
  },
  conversationCreate: (payload) => ipcRenderer.invoke('fraia:conversationCreate', payload),
  conversationConverse: (payload) => ipcRenderer.invoke('fraia:conversationConverse', payload),
  conversationFacts: (payload) => ipcRenderer.invoke('fraia:conversationFacts', payload),
  conversationAgentRespond: (payload) => ipcRenderer.invoke('fraia:conversationAgentRespond', payload),
  conversationCancelDesign: (payload) => ipcRenderer.invoke('fraia:conversationCancelDesign', payload),
  conversationAnalyse: (payload) => ipcRenderer.invoke('fraia:conversationAnalyse', payload),
  startAnalysisAttempt: (payload) => ipcRenderer.invoke('fraia:startAnalysisAttempt', payload),
  analysisAttemptStatus: (payload) => ipcRenderer.invoke('fraia:analysisAttemptStatus', payload),
  cancelAnalysisAttempt: (payload) => ipcRenderer.invoke('fraia:cancelAnalysisAttempt', payload),
  conversationCompare: (payload) => ipcRenderer.invoke('fraia:conversationCompare', payload),
  conversationPropose: (payload) => ipcRenderer.invoke('fraia:conversationPropose', payload),
  conversationAccept: (payload) => ipcRenderer.invoke('fraia:conversationAccept', payload),
  conversationReject: (payload) => ipcRenderer.invoke('fraia:conversationReject', payload),
  conversationFork: (payload) => ipcRenderer.invoke('fraia:conversationFork', payload),
  conversationWorkingCopyOpen: (payload) => ipcRenderer.invoke('fraia:conversationWorkingCopyOpen', payload),
  conversationWorkingCopyApply: (payload) => ipcRenderer.invoke('fraia:conversationWorkingCopyApply', payload),
  conversationWorkingCopyCommit: (payload) => ipcRenderer.invoke('fraia:conversationWorkingCopyCommit', payload),
  refreshProject: (projectDir) => ipcRenderer.invoke('fraia:refreshProject', projectDir),
  refreshProjectIfExists: (projectDir) => ipcRenderer.invoke('fraia:refreshProjectIfExists', projectDir),
  sizeBeam: (payload) => ipcRenderer.invoke('fraia:sizeBeam', payload),
  setThemeSource: (themeSource) => ipcRenderer.invoke('fraia:setThemeSource', themeSource),
  reloadWindow: () => ipcRenderer.invoke('fraia:reloadWindow'),
  forceReloadWindow: () => ipcRenderer.invoke('fraia:forceReloadWindow'),
  quitApp: () => ipcRenderer.invoke('fraia:quitApp'),
});
