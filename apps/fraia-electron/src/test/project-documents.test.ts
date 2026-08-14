import { describe, expect, it } from "vitest"

import {
  projectDocumentFromState,
  projectDocumentLabel,
  reorderProjectDocuments,
  upsertProjectDocument,
} from "@/lib/projectDocuments"
import type { WorkbenchState } from "@/lib/types"

function state(projectDir: string, designName = "Design 1"): WorkbenchState {
  const suffix = projectDir.split("/").pop() ?? "design"
  return {
    overview: {
      projectDir,
      projectId: `project-${suffix}`,
      projectName: `Project ${suffix}`,
      designId: `design-${suffix}`,
      designName,
      documentId: `design-${suffix}`,
      managedUnsaved: false,
    },
    scene: { nodes: [], members: [] },
  }
}

describe("project documents", () => {
  it("uses the stable design id as document identity", () => {
    const document = projectDocumentFromState(state("/projects/frame-a", "Frame A"))

    expect(document).toMatchObject({
      id: "design-frame-a",
      projectDir: "/projects/frame-a",
      label: "Frame A",
      projectId: "project-frame-a",
      designId: "design-frame-a",
    })
  })

  it("falls back to the first design label", () => {
    expect(projectDocumentLabel({ overview: { designName: "" } })).toBe("Design 1")
  })

  it("updates one document without replacing another project state", () => {
    const first = projectDocumentFromState(state("/projects/frame-a"))
    const second = projectDocumentFromState(state("/projects/frame-b"))
    const updatedFirst = projectDocumentFromState({
      ...first.state,
      overview: { ...first.state.overview, projectDir: first.projectDir, designName: "Frame A revised" },
    })

    const documents = upsertProjectDocument([first, second], updatedFirst)

    expect(documents.map((document) => document.label)).toEqual(["Frame A revised", "Design 1"])
    expect(documents[1].state).toBe(second.state)
  })

  it("reorders complete document identities and rejects incomplete orders", () => {
    const first = projectDocumentFromState(state("/projects/frame-a"))
    const second = projectDocumentFromState(state("/projects/frame-b"))

    expect(reorderProjectDocuments([first, second], [second.id, first.id])).toEqual([second, first])
    expect(reorderProjectDocuments([first, second], [first.id])).toEqual([first, second])
  })
})
