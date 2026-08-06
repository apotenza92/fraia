import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Send } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Empty, EmptyContent, EmptyDescription, EmptyTitle } from '@/components/ui/empty';
import { Field, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field';
import { MessageScrollerItem } from '@/components/ui/message-scroller';
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
} from '../chat/ChatTranscript';
import type { AgentProviderStatus, AgentSession, EngineeringScheme, WorkbenchState } from '../../lib/types';
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

const CHAT_INPUT_MAX_LINES = 4;

const SCHEME_CHAT_REPLY_STAGES: AgentProgressStage[] = [
  { label: 'Adding your question to the option context', durationMs: 900 },
  { label: 'Reviewing this design option', durationMs: 2600 },
  { label: 'Checking engineering assumptions', durationMs: 3600 },
  { label: 'Drafting the option-specific answer', durationMs: 4200 },
  { label: 'Preparing reply options', durationMs: 2200 },
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

function compactSchemeContext(scheme: EngineeringScheme) {
  return JSON.stringify({
    id: scheme.id,
    name: scheme.name,
    status: scheme.status,
    supersededBy: scheme.supersededBy,
    supersededReason: scheme.supersededReason,
    revisionOf: scheme.revisionOf,
    summary: scheme.summary,
    recommendation: scheme.recommendation,
    assumptions: scheme.assumptions,
    tradeoffs: scheme.tradeoffs.map((tradeoff) => ({ label: tradeoff.label, compromise: tradeoff.compromise })),
    comparison: scheme.comparison,
  }, null, 2);
}

function withSchemeContext(scheme: EngineeringScheme, text: string) {
  return `${text.trim()}\n\nSelected Fraia engineering design option context for this turn:\n${compactSchemeContext(scheme)}\n\nUse this selected design-option context when answering. Design options are comparison artefacts; do not expose or propose direct Base Model mutation from this chat.`;
}

function displayMessageText(text: string) {
  return text.split('\n\nSelected Fraia engineering design option context for this turn:')[0].split('\n\nSelected Fraia engineering scheme context for this turn:')[0];
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
  const id = `scheme-reply-${label.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`;
  return (
    <Field orientation="horizontal" data-disabled={disabled || undefined} className="gap-2">
      <Checkbox id={id} checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} />
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
    </Field>
  );
}

export function SchemeChatPanel({
  state,
  scheme,
  surface,
  onState,
  showHeader = true,
}: {
  state: WorkbenchState | null;
  scheme: EngineeringScheme;
  surface: string;
  onState: (s: WorkbenchState) => void;
  showHeader?: boolean;
}) {
  const [draft, setDraft] = useState('');
  const [busy, setBusy] = useState(false);
  const [analysing, setAnalysing] = useState(false);
  const [pendingUserText, setPendingUserText] = useState<string | null>(null);
  const [provider, setProvider] = useState<AgentProviderStatus | null>(null);
  const [providerError, setProviderError] = useState<string | null>(null);
  const [analysisError, setAnalysisError] = useState<string | null>(null);
  const [sendError, setSendError] = useState<string | null>(null);
  const [groupedSelections, setGroupedSelections] = useState<Record<string, string[]>>({});
  const [groupedNotes, setGroupedNotes] = useState<Record<string, string>>({});
  const activeRequestIdRef = useRef<string | null>(null);
  const activeRequestTextRef = useRef<string | null>(null);
  const cancelledRequestIdsRef = useRef<Set<string>>(new Set());
  const projectDir = state ? projectDirOf(state) : '';
  const agentState = state?.agentState ?? state?.agent_state;
  const session: AgentSession | undefined = useMemo(() => agentState?.sessions?.find((candidate) => candidate.surface === surface), [agentState, surface]);
  const settings = agentState?.settingsBySurface?.[surface] ?? agentState?.settings_by_surface?.[surface];
  const settingsProviderId = settings?.providerId ?? settings?.provider_id ?? FRAIA_AI_PROVIDER_ID;
  const settingsModelId = settings?.modelId ?? settings?.model_id ?? settings?.model ?? FRAIA_AI_MODEL_ID;
  const selectedModel = selectedAgentModel(provider, settingsProviderId, settingsModelId);
  const aiReady = agentRuntimeReady(provider, selectedModel);
  const showStatus = !aiReady || Boolean(providerError || analysisError || sendError);
  const analysisProgress = useEstimatedAgentProgress(analysing, SCHEME_CHAT_REPLY_STAGES, 'Waiting for option analysis');
  const replyProgress = useEstimatedAgentProgress(busy, SCHEME_CHAT_REPLY_STAGES, 'Design option response');

  const refreshProvider = useCallback(async () => {
    if (!projectDir) return null;
    try {
      setProviderError(null);
      const response = await window.fraia.agentProviderStatus({ projectDir, surface });
      setProvider(response);
      return response as AgentProviderStatus;
    } catch (error: any) {
      setProviderError(error?.message || 'Could not reach the Fraia agent provider.');
      return null;
    }
  }, [projectDir, surface]);

  useEffect(() => {
    setDraft('');
  }, [projectDir, surface]);

  useEffect(() => {
    void refreshProvider();
    return subscribeToAgentModelCatalogRefresh(() => { void refreshProvider(); });
  }, [refreshProvider]);

  async function analyseOption() {
    if (!projectDir || analysing) return;
    setAnalysing(true);
    setAnalysisError(null);
    try {
      const response = await window.fraia.agentStartSession({ projectDir, surface });
      const updated = normalizeWorkbenchState(response);
      if (updated) onState(updated);
    } catch (error: any) {
      setAnalysisError(error?.message || 'Could not analyse this design option.');
    } finally {
      setAnalysing(false);
    }
  }

  async function respond(text: string) {
    const trimmed = text.trim();
    if (!trimmed || !projectDir || busy) return;
    const requestId = createAgentRequestId(surface);
    activeRequestIdRef.current = requestId;
    activeRequestTextRef.current = trimmed;
    setBusy(true);
    setSendError(null);
    setPendingUserText(trimmed);
    setDraft('');
    try {
      const response = await window.fraia.agentRespondSession({ projectDir, surface, sessionId: session?.id, requestId, text: withSchemeContext(scheme, trimmed), selectedOptionIds: [] });
      if (cancelledRequestIdsRef.current.has(requestId)) return;
      const updated = normalizeWorkbenchState(response);
      if (updated) onState(updated);
    } catch (error: any) {
      if (cancelledRequestIdsRef.current.has(requestId)) return;
      setDraft(trimmed);
      setSendError(error?.message || 'Could not send this design-option message.');
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

  const messages = (session?.messages ?? []).filter((message) => !['deterministic', 'local'].includes(message.mode ?? ''));
  const visibleMessages = messages;


  return (
    <div className="flex h-full min-h-0 flex-col gap-0">
      {showHeader ? <div className="flex shrink-0 flex-col gap-2 border-b p-2">
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 items-baseline gap-2">
            <span className="text-sm font-medium">Fraia AI</span>
            <span className="truncate text-xs text-muted-foreground">{FRAIA_AI_MODEL_NAME}</span>
          </div>
          <Badge variant={aiReady ? 'secondary' : 'outline'}>{aiReady ? 'Ready' : 'Sign in required'}</Badge>
        </div>
        {showStatus && (
        <div className="flex flex-col gap-1.5">
        {!aiReady && <Alert><AlertDescription>Open Fraia → Fraia AI and sign in with ChatGPT.</AlertDescription></Alert>}
        {providerError && <Button onClick={refreshProvider} size="sm" variant="secondary">Retry provider check</Button>}
        {analysisError && (
          <Alert variant="destructive">
            <AlertDescription>{analysisError}</AlertDescription>
            <Button onClick={analyseOption} disabled={analysing} size="sm" variant="secondary">Retry option analysis</Button>
          </Alert>
        )}
        {sendError && (
          <Alert variant="destructive">
            <AlertDescription>{sendError}</AlertDescription>
            <Button onClick={() => respond(draft)} disabled={busy || !draft.trim()} size="sm" variant="secondary">Retry send</Button>
          </Alert>
        )}
        </div>
        )}
      </div> : showStatus ? (
        <div className="flex shrink-0 flex-col gap-1.5 p-2">
          {!aiReady && <Alert><AlertDescription>Open Fraia → Fraia AI and sign in with ChatGPT.</AlertDescription></Alert>}
          {providerError && <Button onClick={refreshProvider} size="sm" variant="secondary">Retry provider check</Button>}
          {analysisError && (
            <Alert variant="destructive">
              <AlertDescription>{analysisError}</AlertDescription>
              <Button onClick={analyseOption} disabled={analysing} size="sm" variant="secondary">Retry option analysis</Button>
            </Alert>
          )}
          {sendError && (
            <Alert variant="destructive">
              <AlertDescription>{sendError}</AlertDescription>
              <Button onClick={() => respond(draft)} disabled={busy || !draft.trim()} size="sm" variant="secondary">Retry send</Button>
            </Alert>
          )}
        </div>
      ) : null}

      <div className="min-h-0 flex-1 overflow-hidden">
        <ChatTranscript busy={busy || analysing}>
          {scheme.status === 'superseded' && (
            <MessageScrollerItem><Alert>
              <AlertDescription>
                This design option has been superseded{scheme.supersededBy ? ` by ${scheme.supersededBy}` : ''}. {scheme.supersededReason ?? 'Keep it for review history and compare the replacement option instead.'}
              </AlertDescription>
            </Alert></MessageScrollerItem>
          )}
          {scheme.status === 'rejected' && (
            <MessageScrollerItem><Alert variant="destructive">
              <AlertDescription>
                This design option has been rejected as an active comparison option. {scheme.supersededReason ?? 'Keep it for review history only.'}
              </AlertDescription>
            </Alert></MessageScrollerItem>
          )}
          {!visibleMessages.length && (
            <MessageScrollerItem className="p-px">
              {analysing ? (
                <Card>
                  <CardContent>
                    <EstimatedAgentProgress percent={analysisProgress.percent} stageLabel={analysisProgress.stageLabel} />
                  </CardContent>
                </Card>
              ) : (
                <Empty>
                  <EmptyTitle>No option analysis is available yet.</EmptyTitle>
                  <EmptyDescription>Run option analysis to create the first assistant message for {scheme.name}.</EmptyDescription>
                  <EmptyContent>
                    <Button onClick={analyseOption} disabled={!aiReady} size="sm">Run option analysis</Button>
                  </EmptyContent>
                </Empty>
              )}
            </MessageScrollerItem>
          )}
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
                                disabled={busy || !aiReady}
                                onCheckedChange={() => toggleGroupedReply(groupKey, reply, group.defaultReplies)}
                            />
                          ))}
                        </FieldGroup>
                        <Field>
                        <FieldLabel htmlFor={`${groupKey}-note`} className="sr-only">Other answer or note for {group.title}</FieldLabel>
                        <Textarea
                          id={`${groupKey}-note`}
                          value={groupedNotes[groupKey] ?? ''}
                          placeholder="Add your own answer or note for this issue..."
                          disabled={busy || !aiReady}
                          rows={2}
                          onChange={(event) => setGroupedNotes((current) => ({ ...current, [groupKey]: event.target.value }))}
                        />
                        </Field>
                        </FieldSet>
                        </CardContent>
                      </Card>
                      );
                    })}
                    {visibleMessages[visibleMessages.length - 1] === message && (
                      <Button
                        onClick={() => sendGroupedAnswers(messageKey, replyGroups)}
                        disabled={busy || !aiReady || !groupedAnswerHasContent(messageKey, replyGroups)}
                        className="w-full"
                      >
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
                  </>
                ) : undefined}
              >
                <ChatMessageText text={displayMessageText(message.text)} />
              </ChatTranscriptMessage>
            );
          })}
          {pendingUserText && (
            <>
              <ChatTranscriptMessage author="user" messageId="pending-scheme-user-message">
                <ChatMessageText text={pendingUserText} />
              </ChatTranscriptMessage>
              <ChatTranscriptActivity label="Fraia AI is thinking">
                  <ChatTranscriptCancel onClick={confirmCancelAgentTurn} />
                <EstimatedAgentProgress percent={replyProgress.percent} stageLabel={replyProgress.stageLabel} />
              </ChatTranscriptActivity>
            </>
          )}
        </ChatTranscript>
      </div>

      <Separator />
      <FieldGroup className="shrink-0 gap-2 p-2">
        <Field>
        <FieldLabel htmlFor="scheme-chat-reply" className="sr-only">Ask about {scheme.name}</FieldLabel>
        <Textarea
          id="scheme-chat-reply"
          value={draft}
          placeholder={`Ask about ${scheme.name}...`}
          rows={2}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && event.shiftKey) return;
            if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) respond(draft);
          }}
        />
        </Field>
        <Button onClick={() => respond(draft)} disabled={busy || !draft.trim() || !aiReady} className="w-full">
          {busy ? <Spinner data-icon="inline-start" /> : <Send data-icon="inline-start" />}
          {busy ? 'Sending...' : 'Send'}
        </Button>
      </FieldGroup>
    </div>
  );
}
