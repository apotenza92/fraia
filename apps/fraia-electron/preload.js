const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('fraia', {
  health: () => ipcRenderer.invoke('fraia:health'),
  applicationMetadata: () => ipcRenderer.invoke('fraia:applicationMetadata'),
  defaultProjectDir: () => ipcRenderer.invoke('fraia:defaultProjectDir'),
  createUntitledProject: () => ipcRenderer.invoke('fraia:createUntitledProject'),
  pickProjectFile: () => ipcRenderer.invoke('fraia:pickProjectFile'),
  createProject: (payload) => ipcRenderer.invoke('fraia:createProject', payload),
  openProject: (payload) => ipcRenderer.invoke('fraia:openProject', payload),
  savePlanningDraft: (payload) => ipcRenderer.invoke('fraia:savePlanningDraft', payload),
  materializePlanning: (payload) => ipcRenderer.invoke('fraia:materializePlanning', payload),
  analysePlanning: (payload) => ipcRenderer.invoke('fraia:analysePlanning', payload),
  analyseDesignOptions: (payload) => ipcRenderer.invoke('fraia:analyseDesignOptions', payload),
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
  applyReview: (payload) => ipcRenderer.invoke('fraia:applyReview', payload),
  editBaseModel: (payload) => ipcRenderer.invoke('fraia:editBaseModel', payload),
  conversationCreate: (payload) => ipcRenderer.invoke('fraia:conversationCreate', payload),
  conversationConverse: (payload) => ipcRenderer.invoke('fraia:conversationConverse', payload),
  conversationFacts: (payload) => ipcRenderer.invoke('fraia:conversationFacts', payload),
  conversationAnalyse: (payload) => ipcRenderer.invoke('fraia:conversationAnalyse', payload),
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
  seedFrameDemo: (payload) => ipcRenderer.invoke('fraia:seedFrameDemo', payload),
  seedFrameReviewDemo: (payload) => ipcRenderer.invoke('fraia:seedFrameReviewDemo', payload),
  seedBeamDemo: (payload) => ipcRenderer.invoke('fraia:seedBeamDemo', payload),
  sizeBeam: (payload) => ipcRenderer.invoke('fraia:sizeBeam', payload),
  validateProject: (payload) => ipcRenderer.invoke('fraia:validateProject', payload),
  runFrameCalculix: (payload) => ipcRenderer.invoke('fraia:runFrameCalculix', payload),
  setThemeSource: (themeSource) => ipcRenderer.invoke('fraia:setThemeSource', themeSource),
  reloadWindow: () => ipcRenderer.invoke('fraia:reloadWindow'),
  forceReloadWindow: () => ipcRenderer.invoke('fraia:forceReloadWindow'),
  quitApp: () => ipcRenderer.invoke('fraia:quitApp'),
});
