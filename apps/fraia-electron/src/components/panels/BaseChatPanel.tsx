import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { ClipboardCheck, ExternalLink, LogOut, RotateCcw, Send, Sparkles } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Field, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field';
import { Textarea } from '@/components/ui/textarea';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { Spinner } from '@/components/ui/spinner';
import { Separator } from '@/components/ui/separator';
import { EstimatedAgentProgress, type AgentProgressStage, useEstimatedAgentProgress } from '../chat/AgentProgressIndicator';
import { ChatMessageText } from '../chat/ChatMessageText';
import {
  ChatTranscript,
  ChatTranscriptActivity,
  ChatTranscriptCancel,
  ChatTranscriptMessage,
  ChatTranscriptPanel,
} from '../chat/ChatTranscript';
import type { AgentProviderStatus, AgentSession, BaseModelBrief, WorkbenchState } from '../../lib/types';
import { normalizeWorkbenchState, projectDirOf } from '../../lib/defaultProject';
import {
  FRAIA_AI_MODEL_ID,
  FRAIA_AI_MODEL_NAME,
  FRAIA_AI_PROVIDER_ID,
  agentRuntimeReady,
  selectedAgentModel,
  subscribeToAgentModelCatalogRefresh,
} from '../../lib/agentOptions';
import { CHROME } from '../layout/chromeMetrics';

const SURFACE = 'pre_solve';
const CHAT_INPUT_MAX_LINES = 4;

type AuthenticationEvent = {
  kind?: string;
  flowId?: string;
  providerId?: string;
  type?: string;
  message?: string;
  url?: string;
};

const BASE_GUIDE_START_STAGES: AgentProgressStage[] = [
  { label: 'Reading node and member labels', durationMs: 1200 },
  { label: 'Checking support and load boundaries', durationMs: 2400 },
  { label: 'Grounding structural assumptions', durationMs: 3200 },
  { label: 'Drafting model-specific guide questions', durationMs: 4200 },
  { label: 'Checking grouped reply options', durationMs: 2600 },
];

const BASE_GUIDE_REPLY_STAGES: AgentProgressStage[] = [
  { label: 'Saving your reply into the guide context', durationMs: 1000 },
  { label: 'Reviewing node, member, and constraint context', durationMs: 2600 },
  { label: 'Checking support and load assumptions', durationMs: 3600 },
  { label: 'Updating the Base Model brief', durationMs: 3600 },
  { label: 'Preparing the next grouped questions or handoff', durationMs: 3600 },
];

const DESIGN_OPTION_GENERATION_STAGES: AgentProgressStage[] = [
  { label: 'Reviewing the confirmed Base Model brief', durationMs: 1800 },
  { label: 'Developing distinct structural hypotheses', durationMs: 4200 },
  { label: 'Resolving supports, load paths, and member groups', durationMs: 5200 },
  { label: 'Checking option intent and engineering provenance', durationMs: 4200 },
  { label: 'Building the design-option views', durationMs: 3600 },
];

function messageReplies(message: any): string[] {
  return message.suggestedReplies ?? message.suggested_replies ?? [];
}

function messageReplyGroups(message: any): Array<{ title: string; prompt?: string; replies: string[]; defaultReplies: string[] }> {
  const groups = message.suggestedReplyGroups ?? message.suggested_reply_groups ?? [];
  return groups.map((group: any) => ({
    ...group,
    replies: group.replies ?? [],
    defaultReplies: group.defaultReplies ?? group.default_replies ?? [],
  }));
}

function createAgentRequestId(surface: string) {
  return `${surface}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function briefReady(brief: BaseModelBrief | null) {
  return Boolean(brief?.readiness?.readyForSchemas ?? brief?.readiness?.ready_for_schemas);
}

function isBriefReadyHandoffMessage(message: { author?: string; text?: string; suggestedReplies?: string[]; suggested_replies?: string[] }) {
  const text = (message.text ?? '').toLowerCase();
  const replies = messageReplies(message).join(' ').toLowerCase();
  const combined = `${text} ${replies}`;
  return (
    /\b(design-option intents|apply these design-option intents)\b/.test(combined) ||
    /\bready to generate\b/.test(combined) ||
    /\bgenerate (design options|schemas|schema options)\b/.test(combined) ||
    /\bbase model brief\b[\s\S]*\b(done|ready|complete|confirmed)\b/.test(combined) ||
    /\bready for (design options|schemas|schema generation)\b/.test(combined)
  );
}

function messageIsRenderable(message: { mode?: string }) {
  return !['deterministic', 'local'].includes(message.mode ?? '');
}

function messageStartedGuide(message: { mode?: string }) {
  return message.mode !== 'pi_unavailable';
}

function omissionMeansNoConstraints(group: { title: string; prompt?: string }) {
  const context = `${group.title} ${group.prompt ?? ''}`.toLowerCase();
  return /\bhard constraints?\b|\bno-go\b/.test(context);
}

function agentMessageKey(message: { createdAt?: string; created_at?: string }, index: number) {
  return `${message.createdAt ?? message.created_at ?? index}-${index}`;
}

function CheckboxRow({
  label,
  checked,
  disabled,
  onCheckedChange,
}: {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onCheckedChange: () => void;
}) {
  const id = `base-reply-${label.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`;
  return (
    <Field orientation="horizontal" data-disabled={disabled || undefined} className="gap-2">
      <Checkbox id={id} checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} />
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
    </Field>
  );
}

export function BaseChatPanel({
  state,
  onState,
  onHeaderActionChange,
  onGenerateOptions,
  generatingOptions = false,
  hasDesignOptions = false,
  generationError = null,
}: {
  state: WorkbenchState | null;
  onState: (s: WorkbenchState) => void;
  onHeaderActionChange?: (action: ReactNode | null) => void;
  onGenerateOptions?: () => void | Promise<void>;
  generatingOptions?: boolean;
  hasDesignOptions?: boolean;
  generationError?: string | null;
}) {
  const [draft, setDraft] = useState('');
  const [busy, setBusy] = useState(false);
  const [starting, setStarting] = useState(false);
  const [resettingGuide, setResettingGuide] = useState(false);
  const [pendingUserText, setPendingUserText] = useState<string | null>(null);
  const [provider, setProvider] = useState<AgentProviderStatus | null>(null);
  const [providerError, setProviderError] = useState<string | null>(null);
  const [authenticationAction, setAuthenticationAction] = useState<'sign-in' | 'sign-out' | null>(null);
  const [authenticationError, setAuthenticationError] = useState<string | null>(null);
  const [authenticationEvent, setAuthenticationEvent] = useState<AuthenticationEvent | null>(null);
  const [startError, setStartError] = useState<string | null>(null);
  const [sendError, setSendError] = useState<string | null>(null);
  const [groupedSelections, setGroupedSelections] = useState<Record<string, string[]>>({});
  const [groupedNotes, setGroupedNotes] = useState<Record<string, string>>({});
  const startInFlightRef = useRef(false);
  const activeRequestIdRef = useRef<string | null>(null);
  const activeRequestTextRef = useRef<string | null>(null);
  const cancelledRequestIdsRef = useRef<Set<string>>(new Set());
  const projectDir = state ? projectDirOf(state) : '';
  const agentState = state?.agentState ?? state?.agent_state;
  const session: AgentSession | undefined = useMemo(() => agentState?.sessions?.find((candidate) => candidate.surface === SURFACE), [agentState]);
  const settings = agentState?.settingsBySurface?.[SURFACE] ?? agentState?.settings_by_surface?.[SURFACE];
  const settingsProviderId = settings?.providerId ?? settings?.provider_id ?? FRAIA_AI_PROVIDER_ID;
  const settingsModelId = settings?.modelId ?? settings?.model_id ?? settings?.model ?? FRAIA_AI_MODEL_ID;
  const selectedModel = selectedAgentModel(provider, settingsProviderId, settingsModelId);
  const aiReady = agentRuntimeReady(provider, selectedModel);
  const chatGptProvider = provider?.providers.find((candidate) => candidate.id === FRAIA_AI_PROVIDER_ID);
  const chatGptState = chatGptProvider?.authState ?? chatGptProvider?.auth_state ?? 'disconnected';
  const signedIn = chatGptState === 'connected' || chatGptState === 'configured';
  const oauth = chatGptProvider?.authentication.find((method) => method.type === 'oauth');
  const secureCredentialStorage = provider?.secureCredentialStorageAvailable ?? provider?.secure_credential_storage_available;
  const authenticationInProgress = Boolean(
    authenticationEvent
    && !['complete', 'error'].includes(authenticationEvent.type ?? ''),
  );
  const modelUnavailable = signedIn && !aiReady;
  const messages = (session?.messages ?? []).filter(messageIsRenderable);
  const guideStarted = messages.some(messageStartedGuide);
  const showStatus = Boolean(
    authenticationError
    || authenticationEvent
    || secureCredentialStorage === false
    || modelUnavailable
    || startError
    || providerError
    || sendError,
  );
  const brief = state?.baseModelBrief ?? state?.base_model_brief ?? null;
  const visibleMessages = briefReady(brief)
    ? messages.filter((message) => !isBriefReadyHandoffMessage(message))
    : messages;
  const latestVisibleMessage = visibleMessages[visibleMessages.length - 1];
  const startProgress = useEstimatedAgentProgress(starting, BASE_GUIDE_START_STAGES, 'Finalising node/member-specific guide');
  const replyProgress = useEstimatedAgentProgress(busy, BASE_GUIDE_REPLY_STAGES, 'Finalising the updated Base Model brief');
  const optionProgress = useEstimatedAgentProgress(generatingOptions, DESIGN_OPTION_GENERATION_STAGES, 'Finalising design options');
  const interactionBusy = busy || generatingOptions;
  const refreshProvider = useCallback(async () => {
    if (!projectDir) return null;
    try {
      setProviderError(null);
      const response = await window.fraia.agentProviderStatus({ projectDir, surface: SURFACE });
      setProvider(response);
      return response as AgentProviderStatus;
    } catch (error: any) {
      setProviderError(error?.message || 'Could not reach the Fraia agent provider.');
      return null;
    }
  }, [projectDir]);

  useEffect(() => {
    void refreshProvider();
    return subscribeToAgentModelCatalogRefresh(() => { void refreshProvider(); });
  }, [refreshProvider]);

  useEffect(() => {
    const unsubscribe: unknown = window.fraia.onAiRuntimeStatus?.((event: AuthenticationEvent) => {
      if (event.kind !== 'authentication' || event.providerId !== FRAIA_AI_PROVIDER_ID) return;
      if (event.type === 'complete') {
        setAuthenticationEvent(null);
        void refreshProvider();
      } else {
        setAuthenticationEvent(event);
      }
    });
    return () => {
      if (typeof unsubscribe === 'function') unsubscribe();
    };
  }, [refreshProvider]);

  async function changeAuthentication() {
    if (authenticationAction || authenticationInProgress) return;
    setAuthenticationError(null);
    setAuthenticationEvent(null);

    if (signedIn) {
      setAuthenticationAction('sign-out');
      try {
        await window.fraia.aiDisconnect({ providerId: FRAIA_AI_PROVIDER_ID });
        await refreshProvider();
      } catch (error: any) {
        setAuthenticationError(error?.message || 'Could not sign out of ChatGPT.');
      } finally {
        setAuthenticationAction(null);
      }
      return;
    }

    setAuthenticationAction('sign-in');
    setAuthenticationEvent({
      kind: 'authentication',
      providerId: FRAIA_AI_PROVIDER_ID,
      type: 'progress',
      message: 'Starting ChatGPT sign-in.',
    });
    try {
      await window.fraia.aiStartOAuth({ providerId: FRAIA_AI_PROVIDER_ID });
    } catch (error: any) {
      setAuthenticationEvent(null);
      setAuthenticationError(error?.message || 'Could not start ChatGPT sign-in.');
    } finally {
      setAuthenticationAction(null);
    }
  }

  async function startBaseModelGuide() {
    if (!state || !projectDir || guideStarted || startInFlightRef.current) return;
    startInFlightRef.current = true;
    setStarting(true);
    setStartError(null);
    try {
      const response = await window.fraia.agentStartSession({ projectDir, surface: SURFACE });
      const next = normalizeWorkbenchState(response);
      if (next) onState(next);
      refreshProvider();
    } catch (error: any) {
      setStartError(error?.message || 'Could not start the Base Model chat session.');
    } finally {
      setStarting(false);
      startInFlightRef.current = false;
    }
  }

  const resetBaseModelGuide = useCallback(async () => {
    if (!projectDir || !guideStarted || interactionBusy || starting || resettingGuide) return;
    const confirmed = window.confirm('Reset the Base Model Guide and start again? This clears the Base Model chat and brief, but keeps the current geometry and project model.');
    if (!confirmed) return;
    setResettingGuide(true);
    setStartError(null);
    setSendError(null);
    try {
      const response = await window.fraia.resetBaseModelGuide({ projectDir });
      const updated = normalizeWorkbenchState(response);
      if (updated) onState(updated);
      setDraft('');
      setPendingUserText(null);
      setGroupedSelections({});
      setGroupedNotes({});
      activeRequestIdRef.current = null;
      activeRequestTextRef.current = null;
    } catch (error: any) {
      setSendError(error?.message || 'Could not reset the Base Model Guide.');
    } finally {
      setResettingGuide(false);
    }
  }, [guideStarted, interactionBusy, onState, projectDir, resettingGuide, starting]);

  async function respond(text: string) {
    const trimmed = text.trim();
    if (!trimmed || !projectDir || interactionBusy) return;
    const requestId = createAgentRequestId(SURFACE);
    activeRequestIdRef.current = requestId;
    activeRequestTextRef.current = trimmed;
    setBusy(true);
    setSendError(null);
    setPendingUserText(trimmed);
    setDraft('');
    try {
      const response = await window.fraia.agentRespondSession({ projectDir, surface: SURFACE, sessionId: session?.id, requestId, text: trimmed, selectedOptionIds: [] });
      if (cancelledRequestIdsRef.current.has(requestId)) return;
      const updated = normalizeWorkbenchState(response);
      if (updated) onState(updated);
    } catch (error: any) {
      if (cancelledRequestIdsRef.current.has(requestId)) return;
      setDraft(trimmed);
      setSendError(error?.message || 'Could not send this Base Model message.');
    } finally {
      cancelledRequestIdsRef.current.delete(requestId);
      if (activeRequestIdRef.current === requestId) {
        activeRequestIdRef.current = null;
        activeRequestTextRef.current = null;
        setPendingUserText(null);
        setBusy(false);
      }
    }
  }

  async function confirmCancelAgentTurn() {
    const requestId = activeRequestIdRef.current;
    if (!busy || !requestId) return;
    const confirmed = window.confirm('Cancel this agent response? Fraia will stop the current LLM request.');
    if (!confirmed) return;
    cancelledRequestIdsRef.current.add(requestId);
    const text = activeRequestTextRef.current;
    try {
      await window.fraia.agentCancelSession({ requestId });
      if (text) setDraft(text);
      setPendingUserText(null);
      setBusy(false);
      activeRequestIdRef.current = null;
      activeRequestTextRef.current = null;
    } catch (error: any) {
      cancelledRequestIdsRef.current.delete(requestId);
      setSendError(error?.message || 'Could not cancel this agent response.');
    }
  }

  function sendSuggestedReply(reply: string) {
    respond(reply);
  }

  function selectSuggestedReply(reply: string) {
    setDraft(reply);
  }

  function groupAnswerKey(messageKey: string, groupIndex: number) {
    return `${messageKey}::${groupIndex}`;
  }

  function toggleGroupedReply(groupKey: string, reply: string, defaultReplies: string[] = []) {
    setGroupedSelections((current) => {
      const selected = current[groupKey] ?? defaultReplies;
      const next = selected.includes(reply)
        ? selected.filter((item) => item !== reply)
        : [...selected, reply];
      return { ...current, [groupKey]: next };
    });
  }

  function groupedAnswerText(messageKey: string, groups: Array<{ title: string; replies: string[]; defaultReplies?: string[] }>) {
    return groups
      .map((group, groupIndex) => {
        const key = groupAnswerKey(messageKey, groupIndex);
        const selected = groupedSelections[key] ?? group.defaultReplies ?? [];
        const note = (groupedNotes[key] ?? '').trim();
        const parts = [...selected, note ? `Other detail: ${note}` : ''];
        const answer = parts.filter(Boolean).join(' ');
        if (answer) return `${group.title}: ${answer}`;
        return omissionMeansNoConstraints(group) ? `${group.title}: None` : '';
      })
      .filter(Boolean)
      .join('\n\n');
  }

  function groupedAnswerHasContent(messageKey: string, groups: Array<{ title: string; replies: string[]; defaultReplies?: string[] }>) {
    const hasExplicitAnswer = groups.some((group, groupIndex) => {
      const key = groupAnswerKey(messageKey, groupIndex);
      const selected = groupedSelections[key] ?? group.defaultReplies ?? [];
      return selected.length > 0 || Boolean((groupedNotes[key] ?? '').trim());
    });
    return hasExplicitAnswer || groups.every(omissionMeansNoConstraints);
  }

  function sendGroupedAnswers(messageKey: string, groups: Array<{ title: string; replies: string[]; defaultReplies?: string[] }>) {
    const text = groupedAnswerText(messageKey, groups);
    if (text.trim()) respond(text);
  }


  const statusBanner = (
    <div className="flex shrink-0 flex-col gap-2 border-b p-2">
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-baseline gap-2">
          <span className="text-sm font-medium">Fraia AI</span>
          <span className="truncate text-xs text-muted-foreground">{FRAIA_AI_MODEL_NAME}</span>
        </div>
        <Button
          type="button"
          size="sm"
          variant={signedIn ? 'outline' : 'default'}
          onClick={changeAuthentication}
          disabled={
            authenticationAction !== null
            || authenticationInProgress
            || (!signedIn && (!provider || secureCredentialStorage === false || !oauth))
          }
        >
          {authenticationAction || authenticationInProgress
            ? <Spinner data-icon="inline-start" />
            : signedIn
              ? <LogOut data-icon="inline-start" />
              : <ExternalLink data-icon="inline-start" />}
          {authenticationAction === 'sign-out'
            ? 'Signing out...'
            : authenticationAction === 'sign-in' || authenticationInProgress
              ? 'Waiting for ChatGPT'
              : signedIn
                ? 'Sign out'
                : 'Sign in required'}
        </Button>
      </div>
      {showStatus && (
        <div className="flex flex-col gap-1.5">
      {secureCredentialStorage === false && (
        <Alert variant="destructive">
          <AlertTitle>Secure sign-in unavailable</AlertTitle>
          <AlertDescription>Operating-system credential encryption is unavailable, so Fraia cannot store a ChatGPT authorization.</AlertDescription>
        </Alert>
      )}
      {authenticationError && (
        <Alert variant="destructive">
          <AlertTitle>ChatGPT account action failed</AlertTitle>
          <AlertDescription>{authenticationError}</AlertDescription>
        </Alert>
      )}
      {authenticationEvent?.type === 'auth_url' && (
        <Alert>
          <ExternalLink />
          <AlertTitle>Continue in your browser</AlertTitle>
          <AlertDescription>Fraia opened ChatGPT sign-in in your default browser and will update when authorization finishes.</AlertDescription>
        </Alert>
      )}
      {authenticationEvent?.type === 'progress' && (
        <p className="text-sm text-muted-foreground" role="status">{authenticationEvent.message}</p>
      )}
      {authenticationEvent?.type === 'error' && (
        <Alert variant="destructive">
          <AlertTitle>ChatGPT sign-in failed</AlertTitle>
          <AlertDescription>{authenticationEvent.message}</AlertDescription>
        </Alert>
      )}
      {modelUnavailable && (
        <Alert variant="destructive">
          <AlertTitle>{FRAIA_AI_MODEL_NAME} is unavailable</AlertTitle>
          <AlertDescription>Sign out and reconnect ChatGPT before starting another AI turn.</AlertDescription>
        </Alert>
      )}
      {startError && (
        <Alert variant="destructive">
          <AlertDescription>{startError}</AlertDescription>
          <Button onClick={() => setStartError(null)} size="sm" variant="secondary">Retry chat start</Button>
        </Alert>
      )}
      {providerError && <Button onClick={refreshProvider} size="sm" variant="secondary">Retry provider check</Button>}
      {sendError && (
        <Alert variant="destructive">
          <AlertDescription>{sendError}</AlertDescription>
          <Button onClick={() => respond(draft)} disabled={interactionBusy || !draft.trim()} size="sm" variant="secondary">Retry send</Button>
        </Alert>
      )}
        </div>
      )}
    </div>
  );

  const resetGuideButton = useMemo(() => guideStarted ? (
    <Button
      onClick={resetBaseModelGuide}
      disabled={interactionBusy || starting || resettingGuide}
      className="w-full"
      title="Clear the Base Model Guide chat and brief, while keeping the current model geometry"
    >
      {resettingGuide ? <Spinner data-icon="inline-start" /> : <RotateCcw data-icon="inline-start" />}
      {resettingGuide ? 'Resetting...' : 'Reset guide'}
    </Button>
  ) : null, [guideStarted, interactionBusy, resetBaseModelGuide, resettingGuide, starting]);

  useEffect(() => {
    onHeaderActionChange?.(resetGuideButton);
    return () => onHeaderActionChange?.(null);
  }, [onHeaderActionChange, resetGuideButton]);

  const startGuideButton = (
    <Button
      onClick={startBaseModelGuide}
      disabled={!state || starting || !aiReady}
      className="w-full max-w-[430px]"
    >
      {starting ? <Spinner data-icon="inline-start" /> : <ClipboardCheck data-icon="inline-start" />}
      {starting ? 'Starting Base Model Guide...' : visibleMessages.length ? 'Retry Base Model Guide' : 'Start the Base Model Guide'}
    </Button>
  );

  const composer = (
    <>
    <Separator />
    <FieldGroup className="shrink-0 gap-2 p-2">
      <Field>
      <FieldLabel htmlFor="base-chat-reply" className="sr-only">Reply to the Base Model Guide</FieldLabel>
      <Textarea
        id="base-chat-reply"
        value={draft}
        placeholder="Reply to the Base Model Guide..."
        rows={2}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter' && event.shiftKey) return;
          if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) respond(draft);
        }}
      />
      </Field>
      <div className="grid grid-cols-2 gap-2">
        {resetGuideButton ? <div className="min-w-0">{resetGuideButton}</div> : null}
        <Button className="w-full" onClick={() => respond(draft)} disabled={interactionBusy || !draft.trim() || !aiReady}>
          {busy ? <Spinner data-icon="inline-start" /> : <Send data-icon="inline-start" />}
          {busy ? 'Sending...' : 'Send'}
        </Button>
      </div>
    </FieldGroup>
    </>
  );

  const messageList = (
    <>
      {visibleMessages.map((message, index) => {
        const replies = message.author === 'assistant' ? messageReplies(message) : [];
        const replyGroups = message.author === 'assistant' ? messageReplyGroups(message) : [];
        const messageKey = agentMessageKey(message, index);
        const userMessage = message.author === 'user';
        return (
          <ChatTranscriptMessage
            key={messageKey}
            messageId={messageKey}
            author={userMessage ? 'user' : 'assistant'}
            details={(!!replyGroups.length || !!replies.length) ? (
              <>
              {!!replyGroups.length && (
              <div className="flex flex-col gap-2">
                {replyGroups.map((group, groupIndex) => {
                  const groupKey = groupAnswerKey(messageKey, groupIndex);
                  const selected = groupedSelections[groupKey] ?? group.defaultReplies;
                  return (
                  <Card key={`${group.title}-${group.prompt ?? ''}-${groupIndex}`}>
                    <CardHeader>
                      <CardTitle>{group.title}</CardTitle>
                      {group.prompt && <CardDescription>{group.prompt}</CardDescription>}
                    </CardHeader>
                    <CardContent>
                    <FieldSet className="gap-2">
                    <FieldLegend className="sr-only">{group.title}</FieldLegend>
                    <FieldGroup className="gap-1">
                      {group.replies.map((reply, replyIndex) => (
                        <CheckboxRow
                          key={`${reply}-${replyIndex}`}
                          label={reply}
                            checked={selected.includes(reply)}
                            disabled={interactionBusy || !aiReady}
                            onCheckedChange={() => toggleGroupedReply(groupKey, reply, group.defaultReplies)}
                        />
                      ))}
                    </FieldGroup>
                    <Field>
                    <FieldLabel htmlFor={`${groupKey}-note`} className="sr-only">Other answer or note for {group.title}</FieldLabel>
                    <Textarea
                      id={`${groupKey}-note`}
                      value={groupedNotes[groupKey] ?? ''}
                      rows={2}
                      placeholder="Add your own answer or note for this issue..."
                      disabled={interactionBusy || !aiReady}
                      onChange={(event) => setGroupedNotes((current) => ({ ...current, [groupKey]: event.target.value }))}
                    />
                    </Field>
                    </FieldSet>
                    </CardContent>
                  </Card>
                  );
                })}
                {latestVisibleMessage === message && (
                  <Button onClick={() => sendGroupedAnswers(messageKey, replyGroups)} disabled={interactionBusy || !aiReady || !groupedAnswerHasContent(messageKey, replyGroups)} className="w-full" size="sm">
                    Send selected answers
                  </Button>
                )}
              </div>
            )}
            {!replyGroups.length && !!replies.length && (
              <div className="flex flex-col gap-2">
                {replies.map((reply, replyIndex) => (
                  <div key={`${reply}-${replyIndex}`} className="flex items-stretch gap-2">
                    <Button
                      disabled={interactionBusy || !aiReady}
                      onClick={() => selectSuggestedReply(reply)}
                      variant="outline"
                      className="min-w-0 flex-1 justify-start"
                      size="sm"
                    >
                      {reply}
                    </Button>
                    <Tooltip>
                      <TooltipTrigger
                        render={(
                          <Button
                            disabled={interactionBusy || !aiReady}
                            onClick={() => sendSuggestedReply(reply)}
                            variant="secondary"
                            size="icon-sm"
                            aria-label="Send now"
                          >
                            <Send />
                          </Button>
                        )}
                      />
                      <TooltipContent side="right">Send now</TooltipContent>
                    </Tooltip>
                  </div>
                ))}
              </div>
            )}
              </>
            ) : undefined}
          >
            <ChatMessageText text={message.text} />
          </ChatTranscriptMessage>
        );
      })}
      {pendingUserText && (
        <>
          <ChatTranscriptMessage author="user" messageId="pending-base-user-message">
            <ChatMessageText text={pendingUserText} />
          </ChatTranscriptMessage>
          <ChatTranscriptActivity label="Fraia AI is thinking">
              <ChatTranscriptCancel onClick={confirmCancelAgentTurn} />
            <EstimatedAgentProgress percent={replyProgress.percent} stageLabel={replyProgress.stageLabel} />
          </ChatTranscriptActivity>
        </>
      )}
      {briefReady(brief) && onGenerateOptions && (
        generatingOptions ? (
          <ChatTranscriptActivity label="Fraia AI is generating design options">
            <EstimatedAgentProgress percent={optionProgress.percent} stageLabel={optionProgress.stageLabel} />
          </ChatTranscriptActivity>
        ) : (
          <ChatTranscriptPanel messageId="base-design-options-handoff">
            <Card size="sm" className="w-full">
              <CardHeader>
                <Badge variant="secondary">
                  <Sparkles data-icon="inline-start" />
                  Ready to explore
                </Badge>
                <CardTitle>
                  {hasDesignOptions ? 'Explore a fresh set of design options' : 'Explore design options'}
                </CardTitle>
                <CardDescription>
                  {hasDesignOptions
                    ? 'Your updated Base Model brief is ready. Fraia can develop a fresh set of structural approaches for side-by-side review, while the current set remains available in History.'
                    : 'Your Base Model brief is ready. Fraia can now develop distinct structural approaches for side-by-side review.'}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  You can also keep chatting with the Base Model Guide to refine the model or adjust the brief. Generate options whenever you are ready.
                </p>
                {generationError ? (
                  <Alert variant="destructive" className="mt-3">
                    <AlertDescription>{generationError}</AlertDescription>
                  </Alert>
                ) : null}
              </CardContent>
              <CardFooter>
                <Button
                  type="button"
                  onClick={onGenerateOptions}
                  disabled={interactionBusy}
                  className="w-full"
                  size="sm"
                >
                  <Sparkles data-icon="inline-start" />
                  {hasDesignOptions ? 'Generate new option set' : 'Generate design options'}
                </Button>
              </CardFooter>
            </Card>
          </ChatTranscriptPanel>
        )
      )}
    </>
  );

  const transcript = (
    <ChatTranscript busy={interactionBusy}>
      {messageList}
    </ChatTranscript>
  );

  if (!guideStarted) {
    return (
      <div className="relative flex h-full min-h-0 flex-col gap-0">
        {statusBanner}
        <div className="flex min-h-0 flex-1 items-center justify-center px-8 py-6">
          <div className="flex w-full max-w-[430px] flex-col gap-3">
            {visibleMessages.length > 0 && (
              <div className="h-[42vh] overflow-hidden rounded-md border bg-background/80">
                {transcript}
              </div>
            )}
            {startGuideButton}
            {starting && (
              <Card>
                <CardContent>
                <EstimatedAgentProgress percent={startProgress.percent} stageLabel={startProgress.stageLabel} />
                </CardContent>
              </Card>
            )}
            <p className="text-sm text-muted-foreground">
              Required before Fraia can generate design options. The Base Model is the starting version of your structure; the guide helps Fraia understand what it is, what must stay the same, and what kinds of changes it is allowed to explore.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="relative flex h-full min-h-0 flex-col gap-0">
      {statusBanner}
      <div className="min-h-0 flex-1 overflow-hidden">
        {transcript}
      </div>
      {composer}
    </div>
  );
}
