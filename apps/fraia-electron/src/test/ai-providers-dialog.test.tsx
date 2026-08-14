import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AiProvidersDialog } from '@/components/ai/AiProvidersDialog';

function catalogue(authState: 'connected' | 'disconnected', modelAvailable = authState === 'connected') {
  return {
    providers: [
      {
        id: 'anthropic',
        name: 'Anthropic',
        authentication: [{ type: 'api_key', label: 'Anthropic API key', interactive: true, persistentAllowed: true }],
        authState: 'disconnected',
      },
      {
        id: 'openai-codex',
        name: 'OpenAI Codex',
        authentication: [{ type: 'oauth', label: 'Sign in with ChatGPT', interactive: true, persistentAllowed: true }],
        authState,
        authSource: authState === 'connected' ? 'Encrypted ChatGPT authorization' : null,
      },
    ],
    models: [
      {
        providerId: 'openai-codex',
        modelId: 'gpt-5.6-luna',
        displayName: 'GPT-5.6 Luna',
        available: modelAvailable,
      },
      { providerId: 'anthropic', modelId: 'claude-sonnet', displayName: 'Claude Sonnet', available: false },
    ],
    catalogue: { refreshedAt: '2026-07-27T00:00:00Z', stale: false, source: 'test' },
    secureCredentialStorageAvailable: true,
  };
}

function installFraia(overrides: Record<string, unknown> = {}) {
  Object.defineProperty(window, 'fraia', {
    configurable: true,
    value: {
      aiProviders: vi.fn(async () => catalogue('disconnected')),
      aiRefreshCatalog: vi.fn(async () => catalogue('disconnected')),
      aiDisconnect: vi.fn(async () => catalogue('disconnected')),
      aiStartOAuth: vi.fn(async () => ({ flowId: 'flow-test' })),
      onAiRuntimeStatus: vi.fn(() => () => {}),
      ...overrides,
    },
  });
}

describe('Fraia AI setup', () => {
  it('exposes only the reviewed ChatGPT and Luna contract', async () => {
    const user = userEvent.setup();
    const startOAuth = vi.fn(async () => ({ flowId: 'flow-test' }));
    installFraia({ aiStartOAuth: startOAuth });

    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);

    expect(await screen.findByRole('dialog', { name: 'Fraia AI' })).toBeVisible();
    expect(screen.getByText('ChatGPT')).toBeVisible();
    expect(screen.getByText('Fraia uses one reviewed model for every design conversation.')).toBeVisible();
    expect(screen.queryByText(/gpt-5\.6|reasoning/i)).not.toBeInTheDocument();
    expect(screen.getByText('Sign in required')).toBeVisible();
    expect(screen.queryByText('Anthropic')).not.toBeInTheDocument();
    expect(screen.queryByText('Claude Sonnet')).not.toBeInTheDocument();
    expect(screen.queryByRole('searchbox')).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/API key/i)).not.toBeInTheDocument();
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Sign in with ChatGPT' }));
    await waitFor(() => expect(startOAuth).toHaveBeenCalledWith({ providerId: 'openai-codex' }));
  });

  it('shows encrypted connected status and disconnects ChatGPT', async () => {
    const user = userEvent.setup();
    const disconnect = vi.fn(async () => catalogue('disconnected'));
    installFraia({
      aiProviders: vi.fn(async () => catalogue('connected')),
      aiDisconnect: disconnect,
    });

    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);

    expect(await screen.findByText('Ready')).toBeVisible();
    expect(screen.getByText('Signed in securely. Your authorization stays out of project files.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Sign in with ChatGPT' })).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Disconnect' }));
    await waitFor(() => expect(disconnect).toHaveBeenCalledWith({ providerId: 'openai-codex' }));
    expect(await screen.findByText('Sign in required')).toBeVisible();
  });

  it('fails closed when Luna is unavailable instead of offering a fallback', async () => {
    installFraia({ aiProviders: vi.fn(async () => catalogue('connected', false)) });

    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);

    expect(await screen.findByText('Model unavailable')).toBeVisible();
    expect(screen.getByText('Fraia AI is unavailable')).toBeVisible();
    expect(screen.getByText(/will not switch to another model/i)).toBeVisible();
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('shows one actionable connection sentence and hides technical detail by default', async () => {
    const user = userEvent.setup();
    installFraia({ aiProviders: vi.fn().mockRejectedValue(new Error("Error invoking remote method 'fraia:aiProviders': internal provider schema failure")) });

    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);

    expect(await screen.findByText('Fraia could not finish that action. Try again.')).toBeVisible();
    const details = screen.getByText('Details').closest('details');
    expect(details).not.toHaveAttribute('open');
    expect(details).toHaveTextContent('internal provider schema failure');
    expect(screen.queryByText(/openai-codex|gpt-5\.6-luna/i)).not.toBeInTheDocument();
    await user.click(screen.getByText('Details'));
    expect(details).toHaveAttribute('open');
  });

  it('keeps the primary connection action last in keyboard order', async () => {
    installFraia();
    render(<AiProvidersDialog open onOpenChange={vi.fn()} />);
    await screen.findByText('Sign in required');

    const controls = screen.getAllByRole('button').map((button) => button.textContent?.trim() || button.getAttribute('aria-label'));
    expect(controls.indexOf('Refresh')).toBeLessThan(controls.indexOf('Sign in with ChatGPT'));
    expect(screen.getByRole('dialog')).toContainElement(screen.getByRole('button', { name: 'Close' }));
  });
});
