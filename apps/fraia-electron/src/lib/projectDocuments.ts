import { projectDirOf } from "@/lib/defaultProject"
import type { WorkbenchState } from "@/lib/types"

export type ProjectDocument = {
  id: string
  label: string
  projectDir: string
  state: WorkbenchState
}

export function projectDocumentLabel(state: WorkbenchState) {
  const overview = state.overview
  const explicitName = overview?.fileName
    ?? overview?.file_name
    ?? overview?.workspaceName
    ?? overview?.workspace_name
    ?? overview?.projectName
    ?? overview?.project_name
  if (typeof explicitName === "string" && explicitName.trim()) return explicitName

  const projectDir = projectDirOf(state, "")
  if (projectDir) {
    const parts = projectDir.split(/[\\/]/).filter(Boolean)
    if (parts.length) return parts[parts.length - 1]
  }

  const stateName = overview?.name ?? state.name
  if (typeof stateName === "string" && stateName.trim() && stateName !== "Fraia Electron Workbench") {
    return stateName
  }
  return "Untitled Model"
}

export function projectDocumentFromState(state: WorkbenchState): ProjectDocument {
  const projectDir = projectDirOf(state, "")
  if (!projectDir) throw new Error("Fraia did not return a project location.")
  return {
    id: projectDir,
    label: projectDocumentLabel(state),
    projectDir,
    state,
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
