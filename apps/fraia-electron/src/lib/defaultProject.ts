import type { WorkbenchState } from './types';

export const DEFAULT_PROJECT_DIR = import.meta.env.VITE_FRAIA_DEFAULT_PROJECT_DIR || '/tmp/fraia-electron-raw-cad-geometry-test';
export const DEV_FRESH_GUIDE = import.meta.env.VITE_FRAIA_DEV_FRESH_GUIDE === '1';

export type WorkbenchOperationResponse = {
  message?: string;
  state?: WorkbenchState | null;
  workbench?: WorkbenchState | null;
  [key: string]: any;
};

function isWorkbenchStateLike(value: unknown): value is WorkbenchState {
  if (!value || typeof value !== 'object') return false;
  const candidate = value as WorkbenchState;
  return Boolean(candidate.overview || candidate.scene);
}

export function normalizeWorkbenchState(response: WorkbenchState | WorkbenchOperationResponse | null | undefined): WorkbenchState | null {
  if (!response || typeof response !== 'object') return null;
  if ('state' in response && isWorkbenchStateLike(response.state)) return response.state;
  if ('workbench' in response && isWorkbenchStateLike(response.workbench)) return response.workbench;
  if (isWorkbenchStateLike(response)) return response;
  return null;
}

export function projectDirOf(state: WorkbenchState | null | undefined, fallback = DEFAULT_PROJECT_DIR): string {
  return state?.overview?.projectDir ?? state?.overview?.project_dir ?? state?.projectDir ?? state?.project_dir ?? fallback;
}

export function defaultPlanningRequest(projectDir = DEFAULT_PROJECT_DIR) {
  return { projectDir, draft: {
    projectIntent: { name: 'Raw CAD Geometry Test', buildingType: 'unspecified', designStage: 'concept', objectivePriority: 'analysis readiness' },
    systemBrief: { systemFamilyHint: 'unknown', structuralFormHint: 'raw connected line geometry', notes: 'Default Electron raw geometry smoke model.' },
    geometryAndLoads: {
      span: { value: 18, quantityKind: 'length', canonicalUnit: 'm' },
      height: { value: 6, quantityKind: 'length', canonicalUnit: 'm' },
      gravityLineLoad: { value: 0, quantityKind: 'line_load', canonicalUnit: 'N/m' },
      lateralLoad: { value: 0, quantityKind: 'force', canonicalUnit: 'N' },
    },
    designConstraints: { maxDeflectionRatio: 250, maxDriftRatio: 300, maxUtilization: 0.9, allowInternalColumns: false, maxInternalColumns: 0 },
    analysisBrief: { requestedAnalysisIntent: 'prepare a solver-ready concept model', preferredBackend: null, summaryGoals: 'identify missing assumptions before solving' },
    systemParameters: {}
  }};
}

export function planningRequestFromState(state: WorkbenchState | null | undefined) {
  const projectDir = projectDirOf(state);
  const draft = state?.planningDraft ?? state?.planning_draft;
  if (draft) {
    return { projectDir, draft };
  }
  return defaultPlanningRequest(projectDir);
}

let defaultProjectLoadPromise: Promise<WorkbenchState | null> | null = null;
let defaultProjectDirPromise: Promise<string> | null = null;

async function resolvedDefaultProjectDir() {
  if (import.meta.env.VITE_FRAIA_DEFAULT_PROJECT_DIR) return import.meta.env.VITE_FRAIA_DEFAULT_PROJECT_DIR;
  if (!defaultProjectDirPromise) {
    defaultProjectDirPromise = window.fraia.defaultProjectDir()
      .then((projectDir) => typeof projectDir === 'string' && projectDir.trim() ? projectDir : DEFAULT_PROJECT_DIR)
      .catch(() => DEFAULT_PROJECT_DIR);
  }
  return defaultProjectDirPromise;
}

export async function loadDefaultProject(): Promise<WorkbenchState | null> {
  if (defaultProjectLoadPromise) {
    return defaultProjectLoadPromise;
  }
  defaultProjectLoadPromise = loadDefaultProjectOnce().catch((error) => {
    defaultProjectLoadPromise = null;
    throw error;
  });
  return defaultProjectLoadPromise;
}

async function loadDefaultProjectOnce(): Promise<WorkbenchState | null> {
  if (DEV_FRESH_GUIDE) {
    window.localStorage?.clear();
    window.sessionStorage?.clear();
  }
  const defaultProjectDir = await resolvedDefaultProjectDir();
  let state = normalizeWorkbenchState(await window.fraia.refreshProjectIfExists(defaultProjectDir));
  if (!state) {
    state = normalizeWorkbenchState(await window.fraia.createProject({ projectDir: defaultProjectDir, name: 'Raw CAD Geometry Test' }));
  }
  return state;
}
