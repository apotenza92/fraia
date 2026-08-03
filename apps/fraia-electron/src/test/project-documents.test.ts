import { describe, expect, it } from "vitest"

import {
  projectDocumentFromState,
  projectDocumentLabel,
  reorderProjectDocuments,
  upsertProjectDocument,
} from "@/lib/projectDocuments"
import type { WorkbenchState } from "@/lib/types"

function state(projectDir: string, fileName?: string): WorkbenchState {
  return {
    overview: {
      projectDir,
      ...(fileName ? { fileName } : {}),
    },
    scene: { nodes: [], members: [] },
  }
}

describe("project documents", () => {
  it("uses the canonical project directory as document identity", () => {
    const document = projectDocumentFromState(state("/projects/frame-a", "Frame A"))

    expect(document).toMatchObject({
      id: "/projects/frame-a",
      projectDir: "/projects/frame-a",
      label: "Frame A",
    })
  })

  it("falls back to the project folder for the tab label", () => {
    expect(projectDocumentLabel(state("/projects/frame-b"))).toBe("frame-b")
  })

  it("updates one document without replacing another project state", () => {
    const first = projectDocumentFromState(state("/projects/frame-a"))
    const second = projectDocumentFromState(state("/projects/frame-b"))
    const updatedFirst = projectDocumentFromState({
      ...first.state,
      overview: { ...first.state.overview, projectDir: first.projectDir, fileName: "Frame A revised" },
    })

    const documents = upsertProjectDocument([first, second], updatedFirst)

    expect(documents.map((document) => document.label)).toEqual(["Frame A revised", "frame-b"])
    expect(documents[1].state).toBe(second.state)
  })

  it("reorders complete document identities and rejects incomplete orders", () => {
    const first = projectDocumentFromState(state("/projects/frame-a"))
    const second = projectDocumentFromState(state("/projects/frame-b"))

    expect(reorderProjectDocuments([first, second], [second.id, first.id])).toEqual([second, first])
    expect(reorderProjectDocuments([first, second], [first.id])).toEqual([first, second])
  })
})
