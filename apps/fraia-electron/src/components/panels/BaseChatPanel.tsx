import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { ClipboardCheck, LoaderCircle, RotateCcw, Send } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Textarea } from '@/components/ui/textarea';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { EstimatedAgentProgress, type AgentProgressStage, useEstimatedAgentProgress } from '../chat/AgentProgressIndicator';
import { ChatMessageText } from '../chat/ChatMessageText';
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
    <label htmlFor={id} className="flex items-center gap-2 text-sm">
      <Checkbox id={id} checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} />
      <span>{label}</span>
    </label>
  );
}

export function BaseChatPanel({
  state,
  onState,
  onHeaderActionChange,
}: {
  state: WorkbenchState | null;
  onState: (s: WorkbenchState) => void;
  onHeaderActionChange?: (action: ReactNode | null) => void;
}) {
  const [draft, setDraft] = useState('');
  const [busy, setBusy] = useState(false);
  const [starting, setStarting] = useState(false);
  const [resettingGuide, setResettingGuide] = useState(false);
  const [pendingUserText, setPendingUserText] = useState<string | null>(null);
  const [provider, setProvider] = useState<AgentProviderStatus | null>(null);
  const [providerError, setProviderError] = useState<string | null>(null);
  const [startError, setStartError] = useState<string | null>(null);
  const [sendError, setSendError] = useState<string | null>(null);
  const [groupedSelections, setGroupedSelections] = useState<Record<string, string[]>>({});
  const [groupedNotes, setGroupedNotes] = useState<Record<string, string>>({});
  const startInFlightRef = useRef(false);
  const chatScrollRef = useRef<HTMLDivElement | null>(null);
  const messageRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const latestMessageKeyRef = useRef<string | null>(null);
  const chatPinnedRef = useRef(true);
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
  const messages = (session?.messages ?? []).filter(messageIsRenderable);
  const guideStarted = messages.some(messageStartedGuide);
  const showStatus = Boolean(startError) || (guideStarted && (!aiReady || Boolean(providerError || sendError)));
  const brief = state?.baseModelBrief ?? state?.base_model_brief ?? null;
  const visibleMessages = briefReady(brief)
    ? messages.filter((message) => !isBriefReadyHandoffMessage(message))
    : messages;
  const latestVisibleMessage = visibleMessages[visibleMessages.length - 1];
  const startProgress = useEstimatedAgentProgress(starting, BASE_GUIDE_START_STAGES, 'Finalising node/member-specific guide');
  const replyProgress = useEstimatedAgentProgress(busy, BASE_GUIDE_REPLY_STAGES, 'Finalising the updated Base Model brief');
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
    if (!projectDir || !guideStarted || busy || starting || resettingGuide) return;
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
      latestMessageKeyRef.current = null;
      chatPinnedRef.current = true;
      activeRequestIdRef.current = null;
      activeRequestTextRef.current = null;
    } catch (error: any) {
      setSendError(error?.message || 'Could not reset the Base Model Guide.');
    } finally {
      setResettingGuide(false);
    }
  }, [busy, guideStarted, onState, projectDir, resettingGuide, starting]);

  async function respond(text: string) {
    const trimmed = text.trim();
    if (!trimmed || !projectDir || busy) return;
    const requestId = createAgentRequestId(SURFACE);
    activeRequestIdRef.current = requestId;
    activeRequestTextRef.current = trimmed;
    setBusy(true);
    setSendError(null);
    setPendingUserText(trimmed);
    setDraft('');
    pinChatToBottom();
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
        return answer ? `${group.title}: ${answer}` : '';
      })
      .filter(Boolean)
      .join('\n\n');
  }

  function groupedAnswerHasContent(messageKey: string, groups: Array<{ title: string; replies: string[]; defaultReplies?: string[] }>) {
    return groupedAnswerText(messageKey, groups).trim().length > 0;
  }

  function sendGroupedAnswers(messageKey: string, groups: Array<{ title: string; replies: string[]; defaultReplies?: string[] }>) {
    const text = groupedAnswerText(messageKey, groups);
    if (text.trim()) respond(text);
  }

  function scrollChatToBottom() {
    window.requestAnimationFrame(() => {
      const element = chatScrollRef.current;
      if (element) element.scrollTop = element.scrollHeight;
    });
  }

  function scrollMessageToTop(messageKey: string) {
    window.requestAnimationFrame(() => {
      const container = chatScrollRef.current;
      const messageElement = messageRefs.current[messageKey];
      if (!container || !messageElement) return;
      const containerRect = container.getBoundingClientRect();
      const messageRect = messageElement.getBoundingClientRect();
      container.scrollTop += messageRect.top - containerRect.top - 10;
    });
  }

  function pinChatToBottom() {
    chatPinnedRef.current = true;
    scrollChatToBottom();
  }

  function updateChatPinnedState() {
    const element = chatScrollRef.current;
    if (!element) return;
    const distanceFromBottom = element.scrollHeight - element.scrollTop - element.clientHeight;
    chatPinnedRef.current = distanceFromBottom < 48;
  }

  useEffect(() => {
    const latest = visibleMessages[visibleMessages.length - 1];
    const latestKey = latest ? agentMessageKey(latest, visibleMessages.length - 1) : null;
    const isNewMessage = latestKey !== latestMessageKeyRef.current;
    latestMessageKeyRef.current = latestKey;
    if (isNewMessage && latest?.author === 'assistant' && latestKey) {
      chatPinnedRef.current = false;
      scrollMessageToTop(latestKey);
      return;
    }
    if (chatPinnedRef.current) scrollChatToBottom();
  }, [visibleMessages.length, pendingUserText, busy]);

  const statusBanner = (
    <div className="flex shrink-0 flex-col gap-2 border-b p-2">
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-baseline gap-2">
          <span className="text-sm font-medium">Fraia AI</span>
          <span className="truncate text-xs text-muted-foreground">{FRAIA_AI_MODEL_NAME}</span>
        </div>
        <Badge variant={aiReady ? 'secondary' : 'outline'}>{aiReady ? 'Ready' : 'Sign in required'}</Badge>
      </div>
      {showStatus && (
        <div className="flex flex-col gap-1.5">
      {!aiReady && guideStarted && <Alert><AlertDescription>Open Fraia → Fraia AI and sign in with ChatGPT.</AlertDescription></Alert>}
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
          <Button onClick={() => respond(draft)} disabled={busy || !draft.trim()} size="sm" variant="secondary">Retry send</Button>
        </Alert>
      )}
        </div>
      )}
    </div>
  );

  const resetGuideButton = useMemo(() => guideStarted ? (
    <Button
      onClick={resetBaseModelGuide}
      disabled={busy || starting || resettingGuide}
      className="w-full"
      title="Clear the Base Model Guide chat and brief, while keeping the current model geometry"
    >
      {resettingGuide ? <LoaderCircle /> : <RotateCcw />}
      {resettingGuide ? 'Resetting...' : 'Reset guide'}
    </Button>
  ) : null, [busy, guideStarted, resetBaseModelGuide, resettingGuide, starting]);

  useEffect(() => {
    onHeaderActionChange?.(resetGuideButton);
    return () => onHeaderActionChange?.(null);
  }, [onHeaderActionChange, resetGuideButton]);

  const startGuideButton = (
    <Button
      onClick={startBaseModelGuide}
      disabled={!state || starting}
      className="w-full max-w-[430px]"
    >
      {starting ? <LoaderCircle /> : <ClipboardCheck />}
      {starting ? 'Starting Base Model Guide...' : visibleMessages.length ? 'Retry Base Model Guide' : 'Start the Base Model Guide'}
    </Button>
  );

  const composer = (
    <div className="flex shrink-0 flex-col gap-2 border-t p-2">
      <Textarea
        value={draft}
        placeholder="Reply to the Base Model Guide..."
        rows={2}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter' && event.shiftKey) return;
          if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) respond(draft);
        }}
      />
      <div className="grid grid-cols-2 gap-2">
        {resetGuideButton ? <div className="min-w-0">{resetGuideButton}</div> : null}
        <Button className="w-full" onClick={() => respond(draft)} disabled={busy || !draft.trim() || !aiReady}>
          <Send />
          {busy ? 'Sending...' : 'Send'}
        </Button>
      </div>
    </div>
  );

  const messageList = (
    <div className="flex flex-col gap-2">
      {visibleMessages.map((message, index) => {
        const replies = message.author === 'assistant' ? messageReplies(message) : [];
        const replyGroups = message.author === 'assistant' ? messageReplyGroups(message) : [];
        const messageKey = agentMessageKey(message, index);
        const userMessage = message.author === 'user';
        return (
          <div
            ref={(element) => { messageRefs.current[messageKey] = element; }}
            key={messageKey}
            className={['rounded-md p-2', userMessage ? 'ml-auto max-w-[84%] border shadow-xs' : ''].filter(Boolean).join(' ')}
          >
            <ChatMessageText text={message.text} />
            {!!replyGroups.length && (
              <div className="mt-2 flex flex-col gap-2">
                {replyGroups.map((group, groupIndex) => {
                  const groupKey = groupAnswerKey(messageKey, groupIndex);
                  const selected = groupedSelections[groupKey] ?? group.defaultReplies;
                  return (
                  <Card key={`${group.title}-${group.prompt ?? ''}-${groupIndex}`}>
                    <CardContent className="flex flex-col gap-1">
                    <div className="font-semibold">{group.title}</div>
                    {group.prompt && <p className="text-sm">{group.prompt}</p>}
                    <div className="mt-2 flex flex-col gap-1">
                      {group.replies.map((reply, replyIndex) => (
                        <CheckboxRow
                          key={`${reply}-${replyIndex}`}
                          label={reply}
                            checked={selected.includes(reply)}
                            disabled={busy || !aiReady}
                            onCheckedChange={() => toggleGroupedReply(groupKey, reply, group.defaultReplies)}
                        />
                      ))}
                    </div>
                    <Textarea
                      value={groupedNotes[groupKey] ?? ''}
                      rows={2}
                      placeholder="Add your own answer or note for this issue..."
                      disabled={busy || !aiReady}
                      className="mt-2"
                      onChange={(event) => setGroupedNotes((current) => ({ ...current, [groupKey]: event.target.value }))}
                    />
                    </CardContent>
                  </Card>
                  );
                })}
                {latestVisibleMessage === message && (
                  <Button onClick={() => sendGroupedAnswers(messageKey, replyGroups)} disabled={busy || !aiReady || !groupedAnswerHasContent(messageKey, replyGroups)} className="w-full" size="sm">
                    Send selected answers
                  </Button>
                )}
              </div>
            )}
            {!replyGroups.length && !!replies.length && (
              <div className="mt-2 flex flex-col gap-2">
                {replies.map((reply, replyIndex) => (
                  <div key={`${reply}-${replyIndex}`} className="flex items-stretch gap-2">
                    <Button
                      disabled={busy || !aiReady}
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
                            disabled={busy || !aiReady}
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
          </div>
        );
      })}
      {pendingUserText && (
        <>
          <div className="ml-auto max-w-[84%] rounded-md border p-3 shadow-xs">
            <ChatMessageText text={pendingUserText} />
          </div>
          <Alert>
            <LoaderCircle />
            <AlertTitle>Agent is thinking</AlertTitle>
            <AlertDescription>
              <Button onClick={confirmCancelAgentTurn} variant="secondary" size="sm">
                Cancel response
              </Button>
            <EstimatedAgentProgress percent={replyProgress.percent} stageLabel={replyProgress.stageLabel} />
            </AlertDescription>
          </Alert>
        </>
      )}
    </div>
  );

  if (!guideStarted) {
    return (
      <div className="relative flex h-full min-h-0 flex-col gap-0">
        {statusBanner}
        <div className="flex min-h-0 flex-1 items-center justify-center px-8 py-6">
          <div className="flex w-full max-w-[430px] flex-col gap-3">
            {visibleMessages.length > 0 && (
              <div className="max-h-[42vh] overflow-auto rounded-md border bg-background/80 p-2">
                {messageList}
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
      <div ref={chatScrollRef} onScroll={updateChatPinnedState} className="min-h-0 flex-1 overflow-auto p-2">
        {messageList}
      </div>
      {composer}
    </div>
  );
}
