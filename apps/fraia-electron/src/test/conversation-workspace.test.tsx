import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ConversationWorkspace } from '@/components/conversation/ConversationWorkspace';
import { createConversationProjection, initializeConversation, projectFactsFromState } from '@/lib/conversationWorkspace';
import type { WorkbenchState } from '@/lib/types';

vi.mock('@/components/viewport/Viewport3D', () => ({
  Viewport3D: ({ selectionEnabled = true, scene }: { selectionEnabled?: boolean; scene?: { members?: Array<{ id: string; role?: string }>; nodes?: Array<{ id: string; x: number; y: number; z: number }> } }) => (
    <div data-testid="mock-viewport" data-selection-enabled={String(selectionEnabled)}>
      Structural viewport
      {scene?.members?.map((member) => <span key={member.id} data-testid={`member-role-${member.id}`}>{member.role}</span>)}
      {scene?.nodes?.map((node) => <span key={node.id} data-testid={`node-position-${node.id}`}>{node.x},{node.y},{node.z}</span>)}
    </div>
  ),
}));

const state: WorkbenchState = {
  overview: {
    projectDir: '/tmp/fraia-conversation-test',
    projectId: 'conversation-project',
    projectName: 'Conversation test',
    designId: 'conversation-design',
    designName: 'Design 1',
    documentId: 'conversation-design',
  },
  scene: {
    nodes: [
      { id: 'n1', x: 0, y: 0, z: 0 },
      { id: 'n2', x: 6, y: 0, z: 0 },
    ],
    members: [{ id: 'm1', role: 'rafter', start: 'n1', end: 'n2' }],
    supports: [],
    loads: [],
  },
};

function setConversationBridge(value: Record<string, unknown>) {
  Object.defineProperty(window, 'fraia', { configurable: true, value });
}

function clearConversationBridge() {
  Reflect.deleteProperty(window, 'fraia');
}

describe('conversation workspace', () => {
  it('uses typed projections and contains no staged workflow surface', async () => {
    const projection = createConversationProjection(state);
    expect(projection.projectId).toBe('conversation-project');
    expect(projection.designId).toBe('conversation-design');
    expect(projection.revisionScopeId).toBe('conversation-design');
    expect(projection.head.snapshotId).toBe('root-snapshot');
    expect(projection.artefact.sourceSnapshotId).toBe(projection.head.snapshotId);

    render(<ConversationWorkspace state={state} />);

    expect(screen.getByTestId('conversation-workspace')).toBeVisible();
    expect(screen.getByText('Overall framing')).toBeVisible();
    expect(screen.queryByText('Base Model')).not.toBeInTheDocument();
    expect(screen.queryByText('Design Options')).not.toBeInTheDocument();
    expect(screen.queryByText('Analysis & Comparison')).not.toBeInTheDocument();
    expect(screen.queryByRole('navigation', { name: 'Design workflow' })).not.toBeInTheDocument();
    expect(projection.alternatives).toHaveLength(0);
    expect(screen.queryByTestId('proposal-comparison')).not.toBeInTheDocument();
    expect(screen.queryByTestId('conversation-proposal')).not.toBeInTheDocument();
    expect(screen.queryByTestId('compare-evidence')).not.toBeInTheDocument();
  });

  it('keeps a blank project blank after conversation until a real agent proposal exists', async () => {
    const user = userEvent.setup();
    const blankState: WorkbenchState = {
      overview: {
        projectDir: '/tmp/fraia-blank-conversation-test',
        projectId: 'blank-project',
        projectName: 'Blank conversation',
        designId: 'blank-design',
        designName: 'Design 1',
        documentId: 'blank-design',
      },
      scene: { nodes: [], members: [], supports: [], loads: [] },
    };
    setConversationBridge({
      conversationConverse: vi.fn().mockResolvedValue({
        projectId: 'blank-project',
        conversationId: 'overall-framing',
        purpose: 'Overall framing',
        headRevisionId: 'root-revision',
        headSnapshotId: 'root-snapshot',
        messages: ['Design a supported framing line.'],
      }),
      conversationWorkingCopyOpen: vi.fn().mockResolvedValue({
        workingCopyId: 'blank-working-copy',
        sourceRevisionId: 'root-revision',
        sourceSnapshotId: 'root-snapshot',
      }),
      conversationWorkingCopyApply: vi.fn().mockResolvedValue({}),
    });
    try {
      render(<ConversationWorkspace state={blankState} />);

      expect(screen.getByTestId('blank-conversation')).toBeVisible();
      expect(screen.getByText('What would you like to design?')).toBeVisible();
      expect(screen.queryByTestId('project-brief')).not.toBeInTheDocument();
      expect(screen.queryByTestId('conversation-proposal')).not.toBeInTheDocument();
      expect(document.querySelector('[data-slot="message-scroller-button"]')).not.toBeNull();
      expect(document.querySelector('[data-slot="input-group"]')).not.toBeNull();
      expect(document.querySelector('[data-slot="input-group-control"]')).toBe(screen.getByRole('textbox', { name: 'Conversation message' }));
      expect(screen.getByRole('button', { name: 'Send message' })).toHaveAttribute('aria-disabled', 'true');
      expect(screen.getByRole('button', { name: 'Send message' })).not.toBeDisabled();

      await user.type(screen.getByRole('textbox', { name: 'Conversation message' }), 'Design a supported framing line.');
      expect(screen.getByRole('button', { name: 'Send message' })).toHaveAttribute('aria-disabled', 'false');
      await user.click(screen.getByRole('button', { name: 'Send message' }));

      expect(await screen.findByText('Design a supported framing line.')).toBeVisible();
      expect(screen.getAllByText('Design a supported framing line.')).toHaveLength(1);
      expect(screen.queryByTestId('conversation-proposal')).not.toBeInTheDocument();
      expect(document.querySelector('[data-message-id^="user-"]')).toHaveAttribute('data-scroll-anchor', 'true');
      expect(screen.queryByText('Proposed structure')).not.toBeInTheDocument();
      expect(screen.queryByTestId('member-role-m1')).not.toBeInTheDocument();
    } finally {
      clearConversationBridge();
    }
  });

  it('renders and accepts a reviewed typed agent proposal without proposing it twice', async () => {
    const user = userEvent.setup();
    const propose = vi.fn();
    const listShelf = vi.fn().mockResolvedValue({
      items: {
        confirmed: { id: 'confirmed', label: 'Confirmed plan', confirmation: { confirmed: true } },
        draft: { id: 'draft', label: 'Draft elevation', confirmation: { confirmed: false } },
      },
    });
    const respond = vi.fn().mockResolvedValue({
      responseId: 'typed-response-1',
      text: 'I prepared a traceable supported framing line for review.',
      questions: [],
      proposal: {
        proposalId: 'typed-proposal-1',
        proposedRevisionId: 'typed-revision-1',
        parentRevisionId: 'root-revision',
        assumptions: ['The requested span is six metres.'],
        evidenceLimits: ['No drawing evidence was supplied.'],
        operations: [
          { kind: 'add_node', id: 'left', x: 0, y: 0, z: 0 },
          { kind: 'add_node', id: 'right', x: 6, y: 0, z: 0 },
          { kind: 'add_member', id: 'beam', startNode: 'left', endNode: 'right', role: 'beam', sectionId: '250UB', materialId: 'steel' },
        ],
      },
      provider: 'openai-codex',
      model: 'gpt-5.6-luna',
      reasoningEffort: 'high',
      turnId: 'typed-turn',
    });
    const accept = vi.fn().mockResolvedValue({
      revisionId: 'typed-revision-1',
      snapshotId: 'typed-snapshot-1',
      parentRevisionId: 'root-revision',
      author: 'agent',
      agentProvenance: { provider: 'openai-codex', model: 'gpt-5.6-luna', turnId: 'typed-turn' },
    });
    setConversationBridge({
      listShelf,
      listDrawingInterpretations: vi.fn().mockResolvedValue({ headRevisionId: 'interpretation-aligned', revisions: [] }),
      inspectDrawingInterpretation: vi.fn().mockResolvedValue({
        revisionId: 'interpretation-aligned',
        observations: { grid: { confirmation: { status: 'confirmed' }, designGeometry: { designGeometryKind: 'polyline' } } },
        conflicts: {},
      }),
      conversationAgentRespond: respond,
      conversationPropose: propose,
      conversationAccept: accept,
    });
    try {
      render(<ConversationWorkspace state={{
        ...state,
        overview: {
          ...state.overview,
          projectDir: '/tmp/package/designs/conversation-design',
          projectRootDir: '/tmp/package',
        },
        scene: { nodes: [], members: [], supports: [], loads: [] },
      }} />);
      await user.type(screen.getByRole('textbox', { name: 'Conversation message' }), 'FRAIA_FAKE_TYPED_PROPOSAL_REQUEST');
      await user.click(screen.getByRole('button', { name: 'Send message' }));
      expect(await screen.findByTestId('conversation-proposal')).toBeVisible();
      expect(listShelf).toHaveBeenCalledWith({ projectDir: '/tmp/package', designId: 'conversation-design' });
      expect(respond).toHaveBeenCalledWith(expect.objectContaining({ projectDir: '/tmp/package' }));
      expect(respond).toHaveBeenCalledWith(expect.objectContaining({ shelfItemIds: ['confirmed'] }));
      expect(respond).toHaveBeenCalledWith(expect.objectContaining({ drawingInterpretationRevisionIds: ['interpretation-aligned'] }));
      await user.click(screen.getByRole('button', { name: 'Accept this direction' }));
      expect(accept).toHaveBeenCalledWith(expect.objectContaining({ proposalId: 'typed-proposal-1' }));
      expect(propose).not.toHaveBeenCalled();
      expect(screen.getByTestId('proposal-record')).toBeVisible();
    } finally {
      clearConversationBridge();
    }
  });

  it('shows official turn activity, cancels a pending response, and restores composer focus', async () => {
    const user = userEvent.setup();
    const blankState: WorkbenchState = {
      overview: {
        projectDir: '/tmp/fraia-cancel-response-test',
        projectId: 'cancel-project',
        projectName: 'Cancel response',
        designId: 'cancel-design',
        designName: 'Design 1',
        documentId: 'cancel-design',
      },
      scene: { nodes: [], members: [], supports: [], loads: [] },
    };
    let resolveAgent: ((value: unknown) => void) | undefined;
    const pendingAgent = new Promise((resolve) => { resolveAgent = resolve; });
    const cancel = vi.fn().mockResolvedValue({ status: 'cancelled' });
    setConversationBridge({
      conversationConverse: vi.fn().mockResolvedValue({
        projectId: 'cancel-project',
        conversationId: 'overall-framing',
        purpose: 'Overall framing',
        headRevisionId: 'root-revision',
        headSnapshotId: 'root-snapshot',
        messages: ['Keep this request.'],
      }),
      agentRespondSession: vi.fn().mockReturnValue(pendingAgent),
      agentCancelSession: cancel,
    });
    try {
      render(<ConversationWorkspace state={blankState} />);
      const composer = screen.getByRole('textbox', { name: 'Conversation message' });
      await user.type(composer, 'Keep this request.');
      await user.keyboard('{Control>}{Enter}{/Control}');

      expect(screen.getByRole('status')).toHaveTextContent('Fraia is working…');
      expect(screen.getByRole('button', { name: 'Cancel response' })).toBeVisible();
      expect(document.querySelector('[data-message-id^="agent-activity-"]')).toHaveAttribute('data-scroll-anchor', 'false');

      await user.click(screen.getByRole('button', { name: 'Cancel response' }));
      expect(cancel).toHaveBeenCalledWith(expect.objectContaining({ requestId: expect.stringContaining('conversation-turn-') }));
      expect(screen.getByText('Response cancelled. Your message remains in the conversation.')).toBeVisible();
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
      await waitFor(() => expect(composer).toHaveFocus());

      resolveAgent?.({ state: { ...blankState, agentState: { sessions: [{ surface: 'pre_solve', messages: [{ author: 'assistant', text: 'Late response' }] }] } } });
      await Promise.resolve();
      expect(screen.queryByText('Late response')).not.toBeInTheDocument();
    } finally {
      clearConversationBridge();
    }
  });

  it('keeps a failed structured turn in the open workspace and restores it for Retry', async () => {
    const user = userEvent.setup();
    const message = 'Design a six metre supported framing line.';
    const respond = vi.fn()
      .mockRejectedValueOnce(new Error('structured response remained invalid after one correction'))
      .mockResolvedValueOnce({
        responseId: 'corrected-response',
        text: 'I corrected the structured response. Review this direction.',
        questions: [],
        provider: 'openai-codex',
        model: 'gpt-5.6-luna',
        reasoningEffort: 'high',
        turnId: 'corrected-turn',
      });
    setConversationBridge({ conversationAgentRespond: respond });
    try {
      render(<ConversationWorkspace state={{
        ...state,
        scene: { nodes: [], members: [], supports: [], loads: [] },
      }} />);
      const composer = screen.getByRole('textbox', { name: 'Conversation message' });
      await user.type(composer, message);
      await user.click(screen.getByRole('button', { name: 'Send message' }));

      expect(await screen.findByText(message)).toBeVisible();
      expect(screen.getByText('Fraia could not complete this response. Try again.')).toBeVisible();
      expect(screen.queryByText(/structured response remained invalid/)).not.toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: 'Details' }));
      expect(screen.getByText(/structured response remained invalid/)).toBeVisible();
      expect(screen.getByTestId('conversation-workspace')).toBeVisible();
      await user.click(screen.getByRole('button', { name: 'Try again' }));
      expect(composer).toHaveValue(message);
      expect(composer).toHaveFocus();

      await user.click(screen.getByRole('button', { name: 'Send message' }));
      expect(await screen.findByText('I corrected the structured response. Review this direction.')).toBeVisible();
      expect(respond).toHaveBeenCalledTimes(2);
    } finally {
      clearConversationBridge();
    }
  });

  it('projects the sparse first-use brief into typed conversation facts', () => {
    const projection = createConversationProjection({
      ...state,
      planningDraft: {
        projectIntent: { name: 'Workshop', buildingType: 'workshop', objectivePriority: 'economy' },
        geometryAndLoads: { span: { value: 12, quantityKind: 'length', canonicalUnit: 'm' }, width: 8, height: 4 },
      },
      baseModelBrief: { version: 1, openQuestions: ['Confirm imposed load'] },
    });
    expect(projectFactsFromState({ ...state, planningDraft: { projectIntent: { buildingType: 'workshop' }, geometryAndLoads: { span: 12, width: 8, height: 4 } }, baseModelBrief: { version: 1, openQuestions: ['Confirm imposed load'] } })).toMatchObject({
      buildingType: 'workshop',
      approximateLengthM: 12,
      approximateWidthM: 8,
      approximateHeightM: 4,
      unknowns: ['Confirm imposed load'],
    });
    expect(projection.projectFacts.constraints).toEqual([]);
  });

  it('sends typed first-use facts through conversation creation', async () => {
    const create = vi.fn().mockResolvedValue({ projectId: 'p', conversationId: 'c', purpose: 'Overall framing', headRevisionId: 'root', headSnapshotId: 'snapshot', messages: [] });
    Object.defineProperty(window, 'fraia', { configurable: true, value: { conversationCreate: create } });
    try {
      const projection = createConversationProjection({
        ...state,
        planningDraft: { projectIntent: { buildingType: 'workshop' }, geometryAndLoads: { span: 12, width: 8, height: 4 } },
      });
      await initializeConversation(projection);
      expect(create).toHaveBeenCalledWith(expect.objectContaining({
        projectFacts: expect.objectContaining({ buildingType: 'workshop', approximateLengthM: 12, approximateWidthM: 8, approximateHeightM: 4 }),
      }));
    } finally {
      Reflect.deleteProperty(window, 'fraia');
    }
  });

  it('does not expose analysis before a reviewed typed proposal is accepted', () => {
    const analyse = vi.fn();
    setConversationBridge({
      conversationAnalyse: analyse,
    });
    try {
      render(<ConversationWorkspace state={state} />);
      expect(screen.queryByRole('button', { name: 'Run analysis' })).not.toBeInTheDocument();
      expect(screen.queryByTestId('analysis-result-card')).not.toBeInTheDocument();
      expect(analyse).not.toHaveBeenCalled();
    } finally {
      clearConversationBridge();
    }
  });

  it('shows truthful live analysis stages and a completed canonical run', async () => {
    const user = userEvent.setup();
    const startAnalysisAttempt = vi.fn().mockResolvedValue({
      attemptId: 'attempt-1',
      projectId: 'conversation-design',
      revisionId: 'accepted-revision',
      authoredSnapshotId: 'accepted-snapshot',
      evidenceId: 'analysis-attempt-1',
      stage: 'preparing',
      status: 'running',
      elapsedMillis: 0,
      diagnostics: [],
    });
    const analysisAttemptStatus = vi.fn().mockResolvedValue({
      attemptId: 'attempt-1',
      projectId: 'conversation-design',
      revisionId: 'accepted-revision',
      authoredSnapshotId: 'accepted-snapshot',
      evidenceId: 'analysis-attempt-1',
      stage: 'collecting',
      status: 'completed',
      elapsedMillis: 420,
      canonicalRunId: 'run-1',
      diagnostics: [],
    });
    setConversationBridge({
      conversationCreate: vi.fn().mockResolvedValue({
        projectId: 'conversation-design',
        conversationId: 'overall-framing',
        purpose: 'Overall framing',
        headRevisionId: 'accepted-revision',
        headSnapshotId: 'accepted-snapshot',
        messages: [],
      }),
      startAnalysisAttempt,
      analysisAttemptStatus,
    });
    try {
      render(<ConversationWorkspace state={state} />);
      await user.click(await screen.findByRole('button', { name: 'Run analysis' }));
      await waitFor(() => expect(screen.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'completed'));
      expect(screen.getByTestId('analysis-attempt')).toHaveTextContent('collecting · 0.4 s · completed');
      expect(screen.getAllByText('Analysis complete. The technical record is saved in History.')).toHaveLength(2);
      expect(screen.queryByText(/run-1/)).not.toBeInTheDocument();
      expect(startAnalysisAttempt).toHaveBeenCalledWith(expect.objectContaining({
        projectId: 'conversation-design',
        request: expect.objectContaining({ operation: 'analyse_snapshot' }),
      }));
    } finally {
      clearConversationBridge();
    }
  });

  it('restores an accepted design as an editable artefact after reopen', async () => {
    const blankAcceptedState: WorkbenchState = {
      ...state,
      scene: { nodes: [], members: [], supports: [], loads: [] },
    };
    setConversationBridge({
      conversationCreate: vi.fn().mockResolvedValue({
        projectId: 'conversation-design',
        conversationId: 'overall-framing',
        purpose: 'Overall framing',
        headRevisionId: 'accepted-revision',
        headSnapshotId: 'accepted-snapshot',
        messages: [],
        agentResponses: [{
          responseId: 'accepted-response',
          text: 'Accepted framing proposal.',
          questions: [],
          proposal: {
            proposalId: 'accepted-proposal',
            proposedRevisionId: 'accepted-revision',
            parentRevisionId: 'root-revision',
            status: 'accepted',
            operations: [
              { kind: 'add_node', id: 'left', x: 0, y: 0, z: 0 },
              { kind: 'add_node', id: 'right', x: 6, y: 0, z: 0 },
              { kind: 'add_member', id: 'beam', startNode: 'left', endNode: 'right', role: 'beam' },
            ],
          },
        }],
      }),
    });
    try {
      render(<ConversationWorkspace state={blankAcceptedState} />);
      expect(await screen.findByText('Your current design was restored.')).toBeVisible();
      expect(screen.getAllByRole('button', { name: 'Open in editor' })).not.toHaveLength(0);
    } finally {
      clearConversationBridge();
    }
  });

  it('keeps preview inspection read-only and hands off explicitly to a working copy', async () => {
    const user = userEvent.setup();
    render(<ConversationWorkspace state={state} />);

    expect(screen.getAllByTestId('artefact-preview')[0]).toBeVisible();
    expect(screen.getAllByTestId('mock-viewport')[0]).toHaveAttribute('data-selection-enabled', 'false');

    await user.click(screen.getAllByRole('button', { name: 'Inspect' })[0]);
    expect(screen.getByTestId('artefact-inspection-dialog')).toBeVisible();
    expect(screen.getByText(/Inspection does not edit the model/)).toBeVisible();

    await user.click(within(screen.getByTestId('artefact-inspection-dialog')).getByRole('button', { name: 'Open in editor' }));
    expect(screen.getByTestId('working-copy-panel')).toBeVisible();
    expect(screen.getByText('Edits are private until you return this copy to the conversation.')).toBeVisible();
  });

  it('commits one manual revision projection and exposes compact stale evidence', async () => {
    const user = userEvent.setup();
    const apply = vi.fn().mockResolvedValue({ ok: true });
    setConversationBridge({
      conversationWorkingCopyOpen: vi.fn().mockResolvedValue({
        workingCopyId: 'working-copy-1',
        sourceRevisionId: 'root-revision',
        sourceSnapshotId: 'root-snapshot',
      }),
      conversationWorkingCopyCommit: vi.fn().mockResolvedValue({
        revisionId: 'manual-revision-root-revision',
        snapshotId: 'manual-snapshot',
        parentRevisionId: 'root-revision',
        author: 'manual',
      }),
      conversationWorkingCopyApply: apply,
    });
    try {
      render(<ConversationWorkspace state={state} />);

      await user.click(screen.getAllByRole('button', { name: 'Open in editor' })[0]);
      await user.click(screen.getByRole('button', { name: 'Record manual change' }));
      expect(screen.getByText('1 pending edit')).toBeVisible();
      expect(apply).toHaveBeenCalledWith(expect.objectContaining({
        operation: expect.objectContaining({ kind: 'set_member_role', memberId: expect.any(String) }),
      }));

      await user.click(screen.getByRole('button', { name: 'Return to conversation' }));
      expect(screen.getByTestId('stale-evidence')).toHaveTextContent('Stale evidence');
      expect(screen.getByText(/Your manual changes are back in the conversation/)).toBeVisible();
      expect(screen.getAllByText(/Your manual changes are back in the conversation/)).toHaveLength(1);
      expect(screen.queryByTestId('working-copy-panel')).not.toBeInTheDocument();
    } finally {
      clearConversationBridge();
    }
  });

  it('applies a selected member operation and reflects it in the private preview before commit', async () => {
    const user = userEvent.setup();
    render(<ConversationWorkspace state={state} />);

    await user.click(screen.getAllByRole('button', { name: 'Open in editor' })[0]);
    expect(screen.getByTestId('selected-editor-target')).toHaveTextContent('Selected member m1');
    expect(screen.getByTestId('member-role-m1')).toHaveTextContent('rafter');
    await user.click(screen.getByRole('button', { name: 'Record manual change' }));

    expect(screen.getByTestId('member-role-m1')).toHaveTextContent('beam');
    expect(screen.getByText('1 pending edit')).toBeVisible();
    expect(screen.getByTestId('working-copy-panel')).toBeVisible();
  });

  it('moves the selected node with metre coordinates inside the private preview', async () => {
    const user = userEvent.setup();
    render(<ConversationWorkspace state={state} />);

    await user.click(screen.getAllByRole('button', { name: 'Open in editor' })[0]);
    expect(screen.getByTestId('node-position-n1')).toHaveTextContent('0,0,0');
    await user.clear(screen.getByRole('spinbutton', { name: 'Node x coordinate in metres' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Node x coordinate in metres' }), '1.5');
    await user.click(screen.getByRole('button', { name: 'Move selected node' }));

    expect(screen.getByTestId('node-position-n1')).toHaveTextContent('1.5,0,0');
    expect(screen.getByText('1 pending edit')).toBeVisible();
  });

  it('does not call legacy proposal adapters without a structured agent proposal', () => {
    const propose = vi.fn();
    const accept = vi.fn();
    setConversationBridge({
      conversationPropose: propose,
      conversationAccept: accept,
    });
    try {
      render(<ConversationWorkspace state={state} />);
      expect(screen.queryByRole('button', { name: 'Accept this direction' })).not.toBeInTheDocument();
      expect(screen.queryByTestId('proposal-record')).not.toBeInTheDocument();
      expect(propose).not.toHaveBeenCalled();
      expect(accept).not.toHaveBeenCalled();
    } finally {
      clearConversationBridge();
    }
  });
});
