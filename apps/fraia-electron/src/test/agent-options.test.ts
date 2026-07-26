import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  AGENT_MODEL_CATALOG_REFRESH_INTERVAL_MS,
  selectedAgentModel,
  subscribeToAgentModelCatalogRefresh,
} from '../lib/agentOptions';
import type { AgentProviderStatus } from '../lib/types';

const provider: AgentProviderStatus = {
  providers: [{ id: 'openai-codex', name: 'OpenAI Codex', authentication: [], authState: 'connected' }],
  selectedProviderId: 'openai-codex',
  selectedModelId: 'gpt-current',
  selectedReasoningEffort: 'low',
  diagnostics: [],
  models: [
    {
      providerId: 'openai-codex',
      modelId: 'gpt-current',
      displayName: 'GPT Current',
      defaultReasoningLevel: 'medium',
      supportedReasoningLevels: [],
      available: true,
    },
    {
      providerId: 'openai-codex',
      modelId: 'gpt-new',
      displayName: 'GPT New',
      defaultReasoningLevel: 'medium',
      supportedReasoningLevels: [],
      available: true,
    },
  ],
};

afterEach(() => {
  vi.useRealTimers();
});

describe('agent model catalogue', () => {
  it('preserves an exact user choice and never silently switches a retired model', () => {
    expect(selectedAgentModel(provider, 'openai-codex', 'gpt-new')?.modelId).toBe('gpt-new');
    expect(selectedAgentModel(provider, 'openai-codex', 'retired-model')).toBeUndefined();
  });

  it('refreshes on focus, visibility restoration, and the hourly interval', () => {
    vi.useFakeTimers();
    const refresh = vi.fn();
    const runtimeUnsubscribe = vi.fn();
    let runtimeListener: (() => void) | undefined;
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        onAiRuntimeStatus: (listener: () => void) => {
          runtimeListener = listener;
          return runtimeUnsubscribe;
        },
      },
    });
    const unsubscribe = subscribeToAgentModelCatalogRefresh(refresh);

    window.dispatchEvent(new Event('focus'));
    document.dispatchEvent(new Event('visibilitychange'));
    vi.advanceTimersByTime(AGENT_MODEL_CATALOG_REFRESH_INTERVAL_MS);
    runtimeListener?.();

    expect(refresh).toHaveBeenCalledTimes(4);
    unsubscribe();
    window.dispatchEvent(new Event('focus'));
    vi.advanceTimersByTime(AGENT_MODEL_CATALOG_REFRESH_INTERVAL_MS);
    expect(refresh).toHaveBeenCalledTimes(4);
    expect(runtimeUnsubscribe).toHaveBeenCalledOnce();
  });
});
