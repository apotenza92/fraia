import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AppShell } from '@/components/layout/AppShell';
import type { WorkbenchState } from '@/lib/types';

vi.mock('@/components/conversation/ConversationWorkspace', () => ({
  ConversationWorkspace: () => <div>Conversation</div>,
}));
vi.mock('@/components/sources/ResourceLibrarySheet', () => ({
  ResourceLibrarySheet: () => null,
}));

const state: WorkbenchState = {
  overview: {
    projectDir: '/tmp/fraia-connection',
    projectId: 'project-1',
    projectName: 'House',
    designId: 'design-1',
    designName: 'Main frame',
    documentId: 'design-1',
  },
  scene: { nodes: [], members: [], supports: [], loads: [] },
};

describe('AppShell Fraia connection', () => {
  it('opens the connection dialog from the keyboard-reachable document overflow', async () => {
    const user = userEvent.setup();
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        aiProviders: vi.fn().mockResolvedValue({
          providers: [{ id: 'openai-codex', name: 'ChatGPT', authentication: [{ type: 'oauth' }], authState: 'connected' }],
          models: [{ providerId: 'openai-codex', modelId: 'gpt-5.6-luna', available: true }],
          catalogue: {},
          secureCredentialStorageAvailable: true,
        }),
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });
    render(<AppShell
      state={state}
      onState={vi.fn()}
      documentTabs={[{ id: 'design-1', label: 'Main frame' }]}
      activeDocumentId="design-1"
      onDocumentSelect={vi.fn()}
      onDocumentClose={vi.fn()}
      onDocumentReorder={vi.fn()}
      onOpenDocument={vi.fn()}
      onNewBlankModel={vi.fn()}
      onRenameProject={vi.fn()}
      onRenameDesign={vi.fn()}
      onDeleteDesign={vi.fn()}
      documentActionPending={false}
    />);

    const overflow = screen.getByRole('button', { name: 'Project and design actions' });
    overflow.focus();
    await user.keyboard('{Enter}');
    await user.keyboard('{End}');
    await user.keyboard('{ArrowUp}');
    expect(screen.getByRole('menuitem', { name: 'Fraia connection…' })).toHaveFocus();
    await user.keyboard('{Enter}');

    expect(await screen.findByRole('dialog', { name: 'Fraia AI' })).toBeVisible();
    expect(screen.getByText('Ready')).toBeVisible();
    expect(screen.queryByText(/gpt-5\.6|openai-codex|reasoning/i)).not.toBeInTheDocument();
  });
});
