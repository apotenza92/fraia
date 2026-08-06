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

  it('starts a newly launched Base Model Guide at the top', async () => {
    const startedState: WorkbenchState = {
      overview: { projectDir: '/projects/frame-a' },
      agentState: {
        sessions: [{
          id: 'session-pre-solve',
          surface: 'pre_solve',
          title: 'Base Model Guide',
          status: 'active',
          messages: [{
            author: 'assistant',
            text: 'Start with the model overview, then work through each question in order.',
            mode: 'pi',
          }],
        }],
        settingsBySurface: {
          pre_solve: { providerId: 'openai-codex', modelId: 'gpt-5.6-luna' },
        },
      },
    };
    const agentStartSession = vi.fn(async () => startedState);
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        agentProviderStatus: vi.fn(async () => providerStatus(true)),
        agentStartSession,
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });
    const initialState: WorkbenchState = {
      overview: { projectDir: '/projects/frame-a' },
      agentState: {
        sessions: [],
        settingsBySurface: {
          pre_solve: { providerId: 'openai-codex', modelId: 'gpt-5.6-luna' },
        },
      },
    };
    const onState = vi.fn();
    const user = userEvent.setup();
    const { container, rerender } = render(
      <BaseChatPanel state={initialState} onState={onState} />,
    );

    await user.click(await screen.findByRole('button', { name: 'Start the Base Model Guide' }));
    await waitFor(() => expect(agentStartSession).toHaveBeenCalledWith({
      projectDir: '/projects/frame-a',
      surface: 'pre_solve',
    }));
    await waitFor(() => expect(onState).toHaveBeenCalledWith(startedState));
    rerender(<BaseChatPanel state={startedState} onState={onState} />);

    expect(await screen.findByRole('log')).toBeVisible();
    expect(container.querySelector('[data-slot="message-scroller"]')).toHaveAttribute(
      'data-default-scroll-position',
      'start',
    );
  });

  it('keeps the design-option handoff and generation progress inside the chat', async () => {
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        agentProviderStatus: vi.fn(async () => providerStatus(true)),
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });
    const state: WorkbenchState = {
      overview: { projectDir: '/projects/frame-a' },
      agentState: {
        sessions: [{
          id: 'session-pre-solve',
          surface: 'pre_solve',
          title: 'Base Model Guide',
          status: 'active',
          messages: [{ author: 'assistant', text: 'I have captured the fixed briefing boundaries.', mode: 'pi' }],
        }],
        settingsBySurface: {
          pre_solve: { providerId: 'openai-codex', modelId: 'gpt-5.6-luna' },
        },
      },
      baseModelBrief: {
        version: 1,
        readiness: { readyForSchemas: true },
      },
    };
    const onGenerateOptions = vi.fn();
    const { container, rerender } = render(
      <BaseChatPanel
        state={state}
        onState={vi.fn()}
        onGenerateOptions={onGenerateOptions}
      />,
    );
    const user = userEvent.setup();

    expect(await screen.findByText('Explore design options')).toBeVisible();
    expect(container.querySelector('[data-slot="message-scroller"]')).toHaveAttribute(
      'data-default-scroll-position',
      'last-anchor',
    );
    expect(screen.getByText('Your Base Model brief is ready. Fraia can now develop distinct structural approaches for side-by-side review.')).toBeVisible();
    expect(screen.getByText('You can also keep chatting with the Base Model Guide to refine the model or adjust the brief. Generate options whenever you are ready.')).toBeVisible();
    const generate = screen.getByRole('button', { name: 'Generate design options' });
    expect(generate.closest('[data-slot="message"]')).toHaveAttribute('data-author', 'assistant');
    expect(generate.closest('[data-message-id="base-design-options-handoff"]')).toHaveAttribute('data-scroll-anchor', 'false');
    expect(generate.closest('[data-slot="card"]')?.closest('[data-slot="bubble-content"]')).toBeNull();
    await user.click(generate);
    expect(onGenerateOptions).toHaveBeenCalledOnce();

    rerender(
      <BaseChatPanel
        state={state}
        onState={vi.fn()}
        onGenerateOptions={onGenerateOptions}
        generatingOptions
      />,
    );
    expect(screen.queryByRole('button', { name: 'Generate design options' })).not.toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Fraia AI is generating design options');
    expect(screen.getByText('Reviewing the confirmed Base Model brief')).toBeVisible();
    expect(screen.getByRole('log')).toHaveAttribute('aria-busy', 'true');
  });

  it('treats an omitted hard-constraints answer as none', async () => {
    const agentRespondSession = vi.fn(async () => ({ overview: { projectDir: '/projects/frame-a' } }));
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        agentProviderStatus: vi.fn(async () => providerStatus(true)),
        agentRespondSession,
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });
    const state: WorkbenchState = {
      overview: { projectDir: '/projects/frame-a' },
      agentState: {
        sessions: [{
          id: 'session-pre-solve',
          surface: 'pre_solve',
          title: 'Base Model Guide',
          status: 'active',
          messages: [{
            author: 'assistant',
            text: 'One final boundary question.',
            mode: 'pi',
            suggestedReplyGroups: [{
              title: 'Hard constraints',
              prompt: 'Are there any fixed boundaries or no-go zones?',
              replies: [],
              defaultReplies: [],
            }],
          }],
        }],
        settingsBySurface: {
          pre_solve: { providerId: 'openai-codex', modelId: 'gpt-5.6-luna' },
        },
      },
      baseModelBrief: {
        version: 1,
        readiness: { readyForSchemas: false },
      },
    };
    const user = userEvent.setup();
    const { container } = render(<BaseChatPanel state={state} onState={vi.fn()} />);

    const sendAnswers = await screen.findByRole('button', { name: 'Send selected answers' });
    const groupedReplyCard = container.querySelector('[data-slot="card"]');
    expect(groupedReplyCard).not.toBeNull();
    expect(groupedReplyCard?.querySelector('[data-slot="card-header"]')).not.toBeNull();
    expect(groupedReplyCard?.querySelector('[data-slot="card-title"]')).toHaveTextContent('Hard constraints');
    expect(groupedReplyCard?.closest('[data-slot="bubble-content"]')).toBeNull();
    expect(sendAnswers).toBeEnabled();
    await user.click(sendAnswers);

    await waitFor(() => expect(agentRespondSession).toHaveBeenCalledWith(expect.objectContaining({
      text: 'Hard constraints: None',
    })));
  });
});
