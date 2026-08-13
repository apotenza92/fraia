import { render, screen, within } from '@testing-library/react';
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
  overview: { projectDir: '/tmp/fraia-conversation-test', projectName: 'Conversation test' },
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
    expect(projection.head.snapshotId).toBe('root-snapshot');
    expect(projection.artefact.sourceSnapshotId).toBe(projection.head.snapshotId);

    render(<ConversationWorkspace state={state} />);

    expect(screen.getByTestId('conversation-workspace')).toBeVisible();
    expect(screen.getByText('Overall framing')).toBeVisible();
    expect(screen.queryByText('Base Model')).not.toBeInTheDocument();
    expect(screen.queryByText('Design Options')).not.toBeInTheDocument();
    expect(screen.queryByText('Analysis & Comparison')).not.toBeInTheDocument();
    expect(screen.queryByRole('navigation', { name: 'Design workflow' })).not.toBeInTheDocument();
    expect(projection.alternatives).toHaveLength(2);
    expect(screen.queryByTestId('proposal-comparison')).not.toBeInTheDocument();
    expect(screen.getAllByTestId('conversation-proposal')).toHaveLength(1);
    await userEvent.setup().click(screen.getByRole('button', { name: 'Explore another' }));
    expect(screen.getByTestId('proposal-comparison')).toBeVisible();
    expect(screen.getAllByTestId(/proposal-candidate-/)).toHaveLength(2);
    expect(screen.getByTestId('compare-evidence')).toBeDisabled();
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

  it('runs snapshot-bound analysis and renders a compact evidence result', async () => {
    const user = userEvent.setup();
    setConversationBridge({
      conversationPropose: vi.fn().mockResolvedValue({}),
      conversationAccept: vi.fn().mockResolvedValue({
        revisionId: 'agent-revision-1',
        snapshotId: 'snapshot-1',
        parentRevisionId: 'root-revision',
        author: 'agent',
      }),
      conversationAnalyse: vi.fn().mockResolvedValue({
        evidenceId: 'analysis-root',
        authoredSnapshotId: 'root-snapshot',
        stale: false,
        status: 'success',
        summary: 'Analysis complete',
      }),
    });
    try {
      render(<ConversationWorkspace state={state} />);

      await user.click(screen.getByRole('button', { name: 'Accept this direction' }));
      await user.click(screen.getByRole('button', { name: 'Run analysis' }));
      expect(screen.getByTestId('analysis-result-card')).toBeVisible();
      expect(screen.getAllByText('Analysis complete')).not.toHaveLength(0);
      expect(within(screen.getByTestId('analysis-result-card')).getByText('Bound to the current design')).toBeVisible();
    } finally {
      clearConversationBridge();
    }
  });

  it('keeps preview inspection read-only and hands off explicitly to a working copy', async () => {
    const user = userEvent.setup();
    render(<ConversationWorkspace state={state} />);

    expect(screen.getByTestId('artefact-preview')).toBeVisible();
    expect(screen.getByTestId('mock-viewport')).toHaveAttribute('data-selection-enabled', 'false');

    await user.click(screen.getByRole('button', { name: 'Inspect' }));
    expect(screen.getByTestId('artefact-inspection-dialog')).toBeVisible();
    expect(screen.getByText(/Inspection does not edit the model/)).toBeVisible();

    await user.click(within(screen.getByTestId('artefact-inspection-dialog')).getByRole('button', { name: 'Open in editor' }));
    expect(screen.getByTestId('working-copy-panel')).toBeVisible();
    expect(screen.getByText('Edits are private until you return this copy to the conversation.')).toBeVisible();
  });

  it('commits one manual revision projection and exposes compact stale evidence', async () => {
    const user = userEvent.setup();
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
    });
    try {
      render(<ConversationWorkspace state={state} />);

      await user.click(screen.getByRole('button', { name: 'Open in editor' }));
      await user.click(screen.getByRole('button', { name: 'Record manual change' }));
      expect(screen.getByText('1 pending edit')).toBeVisible();

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

    await user.click(screen.getByRole('button', { name: 'Open in editor' }));
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

    await user.click(screen.getByRole('button', { name: 'Open in editor' }));
    expect(screen.getByTestId('node-position-n1')).toHaveTextContent('0,0,0');
    await user.clear(screen.getByRole('spinbutton', { name: 'Node x coordinate in metres' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Node x coordinate in metres' }), '1.5');
    await user.click(screen.getByRole('button', { name: 'Move selected node' }));

    expect(screen.getByTestId('node-position-n1')).toHaveTextContent('1.5,0,0');
    expect(screen.getByText('1 pending edit')).toBeVisible();
  });

  it('accepts a typed proposal into a new conversation revision projection', async () => {
    const user = userEvent.setup();
    setConversationBridge({
      conversationPropose: vi.fn().mockResolvedValue({}),
      conversationAccept: vi.fn().mockResolvedValue({
        revisionId: 'agent-revision-1',
        snapshotId: 'snapshot-1',
        parentRevisionId: 'root-revision',
        author: 'agent',
        agentProvenance: { provider: 'test-provider', model: 'test-model', turnId: 'test-turn' },
      }),
    });
    try {
      render(<ConversationWorkspace state={state} />);

      await user.click(screen.getByRole('button', { name: 'Accept this direction' }));
      expect(screen.getByText('This direction is now the current design. We can analyse it or refine it.')).toBeVisible();
      expect(screen.getByTestId('proposal-record')).toBeVisible();
      expect(screen.getByText('This direction is now the current design. We can analyse it or refine it.')).toBeVisible();
    } finally {
      clearConversationBridge();
    }
  });
});
