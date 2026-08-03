import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { BaseChatPanel } from '@/components/panels/BaseChatPanel';
import type { AgentProviderStatus, WorkbenchState } from '@/lib/types';

function providerStatus(connected: boolean): AgentProviderStatus {
  return {
    providers: [{
      id: 'openai-codex',
      name: 'OpenAI Codex',
      authentication: [{
        type: 'oauth',
        label: 'Sign in with ChatGPT',
        interactive: true,
        persistentAllowed: true,
      }],
      authState: connected ? 'connected' : 'disconnected',
    }],
    models: [{
      providerId: 'openai-codex',
      modelId: 'gpt-5.6-luna',
      displayName: 'GPT-5.6 Luna',
      available: connected,
    }],
    selectedProviderId: 'openai-codex',
    selectedModelId: 'gpt-5.6-luna',
    secureCredentialStorageAvailable: true,
  };
}

describe('Base Model ChatGPT authentication', () => {
  it('starts sign-in from the main window and becomes a sign-out control', async () => {
    const listeners = new Set<(event: Record<string, unknown>) => void>();
    let connected = false;
    const agentProviderStatus = vi.fn(async () => providerStatus(connected));
    const aiStartOAuth = vi.fn(async () => {
      connected = true;
      listeners.forEach((listener) => listener({
        kind: 'authentication',
        providerId: 'openai-codex',
        flowId: 'flow-test',
        type: 'complete',
      }));
      return { flowId: 'flow-test' };
    });
    const aiDisconnect = vi.fn(async () => {
      connected = false;
      return providerStatus(false);
    });

    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        agentProviderStatus,
        aiStartOAuth,
        aiDisconnect,
        onAiRuntimeStatus: vi.fn((listener) => {
          listeners.add(listener);
          return () => listeners.delete(listener);
        }),
      },
    });

    const state: WorkbenchState = {
      overview: { projectDir: '/projects/frame-a' },
      agentState: {
        sessions: [],
        settingsBySurface: {
          pre_solve: {
            providerId: 'openai-codex',
            modelId: 'gpt-5.6-luna',
          },
        },
      },
    };
    const user = userEvent.setup();
    render(<BaseChatPanel state={state} onState={vi.fn()} />);

    const signIn = await screen.findByRole('button', { name: 'Sign in required' });
    expect(screen.getByRole('button', { name: 'Start the Base Model Guide' })).toBeDisabled();
    await user.click(signIn);

    await waitFor(() => expect(aiStartOAuth).toHaveBeenCalledWith({ providerId: 'openai-codex' }));
    expect(await screen.findByRole('button', { name: 'Sign out' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Start the Base Model Guide' })).toBeEnabled();

    await user.click(screen.getByRole('button', { name: 'Sign out' }));

    await waitFor(() => expect(aiDisconnect).toHaveBeenCalledWith({ providerId: 'openai-codex' }));
    expect(await screen.findByRole('button', { name: 'Sign in required' })).toBeVisible();
  });
});
