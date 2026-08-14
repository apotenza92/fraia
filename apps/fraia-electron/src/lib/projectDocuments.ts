import { projectDirOf } from "@/lib/defaultProject"
import type { WorkbenchState } from "@/lib/types"

export type ProjectDocument = {
  id: string
  label: string
  projectDir: string
  projectRootDir: string
  projectId: string
  projectName: string
  designId: string
  designName: string
  managedUnsaved: boolean
  state: WorkbenchState
}

export function projectDocumentLabel(state: WorkbenchState) {
  const overview = state.overview
  const explicitName = overview?.designName ?? overview?.design_name
  if (typeof explicitName === "string" && explicitName.trim()) return explicitName
  return "Design 1"
}

export function projectDocumentFromState(state: WorkbenchState): ProjectDocument {
  const projectDir = projectDirOf(state, "")
  if (!projectDir) throw new Error("Fraia did not return a project location.")
  const projectId = state.overview?.projectId ?? state.overview?.project_id
  const projectName = state.overview?.projectName ?? state.overview?.project_name
  const designId = state.overview?.designId ?? state.overview?.design_id ?? state.overview?.documentId ?? state.overview?.document_id
  const designName = state.overview?.designName ?? state.overview?.design_name
  const projectRootDir = state.overview?.projectRootDir ?? state.overview?.project_root_dir ?? projectDir
  if (![projectId, projectName, designId, designName].every((value) => typeof value === "string" && value.trim())) {
    throw new Error("Fraia did not return stable project and design identity.")
  }
  return {
    id: designId,
    label: designName,
    projectDir,
    projectRootDir,
    projectId,
    projectName,
    designId,
    designName,
    managedUnsaved: state.overview?.managedUnsaved === true,
    state,
  }
}

export function preserveProjectIdentity(nextState: WorkbenchState, current: ProjectDocument): WorkbenchState {
  return {
    ...nextState,
    overview: {
      ...nextState.overview,
      projectDir: projectDirOf(nextState, current.projectDir),
      projectRootDir: current.projectRootDir,
      projectId: current.projectId,
      projectName: current.projectName,
      designId: current.designId,
      designName: current.designName,
      documentId: current.designId,
      managedUnsaved: current.managedUnsaved,
    },
  }
}

export function upsertProjectDocument(
  documents: ProjectDocument[],
  document: ProjectDocument,
) {
  const existingIndex = documents.findIndex((candidate) => candidate.id === document.id)
  if (existingIndex < 0) return [...documents, document]
  return documents.map((candidate, index) => index === existingIndex ? document : candidate)
}

export function reorderProjectDocuments(
  documents: ProjectDocument[],
  orderedIds: string[],
) {
  if (documents.length !== orderedIds.length || new Set(orderedIds).size !== orderedIds.length) {
    return documents
  }
  const byId = new Map(documents.map((document) => [document.id, document]))
  const reordered = orderedIds.map((id) => byId.get(id))
  return reordered.every((document): document is ProjectDocument => Boolean(document))
    ? reordered
    : documents
}
