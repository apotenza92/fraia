import type { AgentModelOption, AgentProviderStatus, AgentReasoningOption } from './types';

export const AGENT_MODEL_CATALOG_REFRESH_INTERVAL_MS = 60 * 60 * 1000;
export const FRAIA_AI_PROVIDER_ID = 'openai-codex';
export const FRAIA_AI_MODEL_ID = 'gpt-5.6-luna';
export const FRAIA_AI_MODEL_NAME = 'GPT-5.6 Luna';
export const FRAIA_AI_REASONING_EFFORT = 'low';

const OFF_REASONING_OPTION: AgentReasoningOption = {
  effort: 'off',
  description: 'Disable model reasoning',
};

type ReasoningOptionSettings = {
  allowOff?: boolean;
};

export function reasoningOptionsForModel(model?: AgentModelOption, settings: ReasoningOptionSettings = {}): AgentReasoningOption[] {
  const options = model?.supportedReasoningLevels ?? model?.supported_reasoning_levels ?? [];
  const allowOff = settings.allowOff ?? true;
  if (!allowOff) {
    return options.filter((option) => option.effort !== OFF_REASONING_OPTION.effort);
  }
  return options.some((option) => option.effort === OFF_REASONING_OPTION.effort)
    ? options
    : [OFF_REASONING_OPTION, ...options];
}

export function selectedAgentModel(
  provider: AgentProviderStatus | null,
  requestedProvider?: string,
  requestedModel?: string,
): AgentModelOption | undefined {
  const models = provider?.models ?? [];
  const providerSelection = provider?.selectedProviderId ?? provider?.selected_provider_id;
  const modelSelection = provider?.selectedModelId ?? provider?.selected_model_id ?? provider?.selectedModel ?? provider?.selected_model;
  const providerId = requestedProvider ?? providerSelection;
  const modelId = requestedModel ?? modelSelection;
  return models.find((model) => agentModelProviderId(model) === providerId && agentModelId(model) === modelId);
}

export function agentModelId(model: AgentModelOption) {
  return model.modelId ?? model.model_id ?? model.slug ?? '';
}

export function agentModelProviderId(model: AgentModelOption) {
  return model.providerId ?? model.provider_id ?? '';
}

export function availableAgentModels(provider: AgentProviderStatus | null) {
  return (provider?.models ?? []).filter((model) => model.available !== false);
}

export function agentRuntimeReady(provider: AgentProviderStatus | null, selected?: AgentModelOption) {
  if (!selected || selected.available === false) return false;
  const providerId = agentModelProviderId(selected);
  const descriptor = provider?.providers?.find((candidate) => candidate.id === providerId);
  const authState = descriptor?.authState ?? descriptor?.auth_state;
  return authState === 'connected' || authState === 'configured';
}

export function subscribeToAgentModelCatalogRefresh(refresh: () => void) {
  const refreshWhenVisible = () => {
    if (document.visibilityState === 'visible') refresh();
  };
  window.addEventListener('focus', refreshWhenVisible);
  document.addEventListener('visibilitychange', refreshWhenVisible);
  const intervalId = window.setInterval(refreshWhenVisible, AGENT_MODEL_CATALOG_REFRESH_INTERVAL_MS);
  const runtimeUnsubscribe: unknown = window.fraia?.onAiRuntimeStatus?.(() => refresh());
  return () => {
    window.removeEventListener('focus', refreshWhenVisible);
    document.removeEventListener('visibilitychange', refreshWhenVisible);
    window.clearInterval(intervalId);
    if (typeof runtimeUnsubscribe === 'function') runtimeUnsubscribe();
  };
}
