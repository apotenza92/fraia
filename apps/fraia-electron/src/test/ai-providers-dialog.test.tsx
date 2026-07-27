import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AiProvidersDialog } from '@/components/ai/AiProvidersDialog';

function catalogue(authState: 'connected' | 'disconnected') {
  return {
    providers: [
      {
        id: 'fraia-test',
        name: 'Fraia Test Provider',
        authentication: [{ type: 'api_key', label: 'Test API key', interactive: true, persistentAllowed: true }],
        authState,
        authSource: authState === 'connected' ? 'encrypted test credential' : null,
      },
      {
        id: 'cloud-test',
        name: 'Cloud Test Provider',
        authentication: [{ type: 'external', label: 'Cloud profile', requirements: ['CLOUD_PROFILE'] }],
        authState: 'disconnected',
      },
      {
        id: 'openai-codex-test',
        name: 'OpenAI Codex',
        authentication: [{ type: 'oauth', label: 'OpenAI (ChatGPT Plus/Pro)', interactive: true, persistentAllowed: true }],
        authState: 'disconnected',
      },
    ],
    models: [
      { providerId: 'fraia-test', modelId: 'model-a', available: authState === 'connected' },
      { providerId: 'openai-codex-test', modelId: 'gpt-test', available: false },
    ],
    catalogue: { refreshedAt: '2026-07-22T00:00:00Z', stale: false, source: 'test' },
    secureCredentialStorageAvailable: true,
  };
}

describe('AI providers settings', () => {
  it('masks and clears API keys while exposing external configuration requirements', async () => {
    const user = userEvent.setup();
    const submitApiKey = vi.fn(async (payload: { providerId: string; apiKey: string }) => {
      expect(payload).toEqual({ providerId: 'fraia-test', apiKey: 'one-use-secret' });
      return catalogue('connected');
    });
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        aiProviders: vi.fn(async () => catalogue('disconnected')),
        aiSubmitApiKey: submitApiKey,
        aiRefreshCatalog: vi.fn(async () => catalogue('disconnected')),
        aiDisconnect: vi.fn(async () => catalogue('disconnected')),
        aiStartOAuth: vi.fn(),
        aiAnswerAuthPrompt: vi.fn(),
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });

    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);
    const input = await screen.findByLabelText('Test API key');
    expect(input).toHaveAttribute('type', 'password');
    expect(screen.getByText('Cloud profile: CLOUD_PROFILE')).toBeVisible();
    expect(document.querySelector('[data-slot="card"]')).not.toBeInTheDocument();
    expect(screen.getAllByRole('listitem')).toHaveLength(3);

    await user.type(input, 'one-use-secret');
    await user.click(screen.getByRole('button', { name: 'Connect' }));

    await waitFor(() => expect(submitApiKey).toHaveBeenCalledOnce());
    expect(screen.queryByDisplayValue('one-use-secret')).not.toBeInTheDocument();
    expect(await screen.findByText('encrypted test credential')).toBeVisible();
  });

  it('starts ChatGPT subscription authentication from the provider list', async () => {
    const user = userEvent.setup();
    const startOAuth = vi.fn(async () => ({ flowId: 'flow-test' }));
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        aiProviders: vi.fn(async () => catalogue('disconnected')),
        aiSubmitApiKey: vi.fn(),
        aiRefreshCatalog: vi.fn(async () => catalogue('disconnected')),
        aiDisconnect: vi.fn(async () => catalogue('disconnected')),
        aiStartOAuth: startOAuth,
        aiAnswerAuthPrompt: vi.fn(),
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });

    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);
    await user.type(await screen.findByLabelText('Search providers'), 'ChatGPT');
    expect(screen.queryByText('Fraia Test Provider')).not.toBeInTheDocument();
    expect(screen.getByText('OpenAI Codex')).toBeVisible();
    await user.click(await screen.findByRole('button', { name: 'OpenAI (ChatGPT Plus/Pro)' }));

    await waitFor(() => expect(startOAuth).toHaveBeenCalledWith({ providerId: 'openai-codex-test' }));
  });
});
