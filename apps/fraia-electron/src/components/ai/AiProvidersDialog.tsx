import { useCallback, useEffect, useMemo, useState } from 'react';
import { ExternalLink, LoaderCircle, RefreshCw, ShieldCheck, Sparkles, Unplug } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { Field, FieldContent, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemMedia,
  ItemTitle,
} from '@/components/ui/item';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import {
  FRAIA_AI_MODEL_ID,
  FRAIA_AI_MODEL_NAME,
  FRAIA_AI_PROVIDER_ID,
  FRAIA_AI_REASONING_EFFORT,
} from '@/lib/agentOptions';
import { cn } from '@/lib/utils';
import type { AiProviderCatalogue } from '@/lib/types';

type RuntimeEvent = {
  kind?: string;
  flowId?: string;
  providerId?: string;
  type?: string;
  message?: string;
  url?: string;
  userCode?: string;
  verificationUri?: string;
  prompt?: { type?: string; message?: string; options?: Array<{ id: string; label: string }> };
};

function providerState(provider: NonNullable<AiProviderCatalogue['providers']>[number]) {
  return provider.authState ?? provider.auth_state ?? 'disconnected';
}

export function AiProvidersDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [catalogue, setCatalogue] = useState<AiProviderCatalogue | null>(null);
  const [busyAction, setBusyAction] = useState<'refresh' | 'sign-in' | 'disconnect' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [runtimeEvent, setRuntimeEvent] = useState<RuntimeEvent | null>(null);
  const [promptAnswer, setPromptAnswer] = useState('');

  const load = useCallback(async () => {
    setError(null);
    try {
      setCatalogue(await window.fraia.aiProviders());
    } catch (cause: any) {
      setError(cause?.message || 'Could not load Fraia AI setup.');
    }
  }, []);

  useEffect(() => {
    if (open) void load();
  }, [load, open]);

  useEffect(() => {
    const unsubscribe: unknown = window.fraia.onAiRuntimeStatus?.((event: RuntimeEvent) => {
      if (event.kind !== 'authentication' || event.providerId !== FRAIA_AI_PROVIDER_ID) return;
      setRuntimeEvent(event);
      if (event.type === 'complete') void load();
    });
    return () => {
      if (typeof unsubscribe === 'function') unsubscribe();
    };
  }, [load]);

  const chatGptProvider = useMemo(
    () => catalogue?.providers.find((provider) => provider.id === FRAIA_AI_PROVIDER_ID),
    [catalogue],
  );
  const lunaModel = useMemo(
    () => catalogue?.models.find((model) => (
      (model.providerId ?? model.provider_id) === FRAIA_AI_PROVIDER_ID
      && (model.modelId ?? model.model_id ?? model.slug) === FRAIA_AI_MODEL_ID
    )),
    [catalogue],
  );
  const state = chatGptProvider ? providerState(chatGptProvider) : 'disconnected';
  const connected = state === 'connected' || state === 'configured';
  const lunaReady = connected && lunaModel?.available !== false && Boolean(lunaModel);
  const oauth = chatGptProvider?.authentication.find((method) => method.type === 'oauth');
  const authInProgress = Boolean(runtimeEvent && !['complete', 'error'].includes(runtimeEvent.type ?? ''));

  async function startOAuth() {
    setBusyAction('sign-in');
    setError(null);
    setRuntimeEvent(null);
    try {
      await window.fraia.aiStartOAuth({ providerId: FRAIA_AI_PROVIDER_ID });
    } catch (cause: any) {
      setError(cause?.message || 'Could not start ChatGPT sign-in.');
    } finally {
      setBusyAction(null);
    }
  }

  async function disconnect() {
    setBusyAction('disconnect');
    setError(null);
    setRuntimeEvent(null);
    try {
      setCatalogue(await window.fraia.aiDisconnect({ providerId: FRAIA_AI_PROVIDER_ID }));
    } catch (cause: any) {
      setError(cause?.message || 'Could not disconnect ChatGPT.');
    } finally {
      setBusyAction(null);
    }
  }

  async function refresh() {
    setBusyAction('refresh');
    setError(null);
    try {
      setCatalogue(await window.fraia.aiRefreshCatalog());
    } catch (cause: any) {
      setError(cause?.message || 'Could not refresh Fraia AI.');
    } finally {
      setBusyAction(null);
    }
  }

  async function answerPrompt(event: RuntimeEvent) {
    if (!event.flowId) return;
    await window.fraia.aiAnswerAuthPrompt({ flowId: event.flowId, value: promptAnswer });
    setPromptAnswer('');
  }

  const secureStorage = catalogue?.secureCredentialStorageAvailable ?? catalogue?.secure_credential_storage_available;
  const freshness = catalogue?.catalogue?.refreshedAt ?? catalogue?.catalogue?.refreshed_at;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Fraia AI</DialogTitle>
          <DialogDescription>
            Sign in with ChatGPT to use Fraia&apos;s guided engineering workflows. Fraia keeps the authorization encrypted by your operating system and out of project files.
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-4">
          {secureStorage === false && (
            <Alert variant="destructive">
              <ShieldCheck />
              <AlertTitle>Secure sign-in unavailable</AlertTitle>
              <AlertDescription>
                Operating-system credential encryption is unavailable, so Fraia will not store a ChatGPT authorization.
              </AlertDescription>
            </Alert>
          )}
          {error && (
            <Alert variant="destructive">
              <AlertTitle>Fraia AI could not finish that action</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {catalogue && !chatGptProvider && (
            <Empty>
              <EmptyHeader>
                <EmptyMedia variant="icon"><Sparkles /></EmptyMedia>
                <EmptyTitle>ChatGPT sign-in is unavailable</EmptyTitle>
                <EmptyDescription>
                  The installed Pi runtime did not provide Fraia&apos;s reviewed ChatGPT connection.
                </EmptyDescription>
              </EmptyHeader>
            </Empty>
          )}

          {chatGptProvider && (
            <>
              <ItemGroup>
                <Item variant="muted">
                  <ItemMedia variant="icon"><Sparkles /></ItemMedia>
                  <ItemContent>
                    <ItemTitle>ChatGPT</ItemTitle>
                    <ItemDescription>
                      {FRAIA_AI_MODEL_NAME} · {FRAIA_AI_REASONING_EFFORT[0].toUpperCase() + FRAIA_AI_REASONING_EFFORT.slice(1)} reasoning
                    </ItemDescription>
                  </ItemContent>
                  <ItemActions>
                    <Badge variant={lunaReady ? 'secondary' : 'outline'}>
                      {lunaReady ? 'Ready' : connected ? 'Model unavailable' : 'Sign in required'}
                    </Badge>
                  </ItemActions>
                </Item>
              </ItemGroup>

              {connected && (
                <p className="text-sm text-muted-foreground">
                  {chatGptProvider.authSource ?? chatGptProvider.auth_source ?? 'ChatGPT authorization is connected.'}
                </p>
              )}
              {connected && !lunaReady && (
                <Alert variant="destructive">
                  <AlertTitle>{FRAIA_AI_MODEL_NAME} is unavailable</AlertTitle>
                  <AlertDescription>
                    Fraia does not silently switch models. Refresh the catalogue or reconnect ChatGPT before starting another AI turn.
                  </AlertDescription>
                </Alert>
              )}
              {runtimeEvent?.type === 'device_code' && (
                <Alert>
                  <ExternalLink />
                  <AlertTitle>Finish signing in with ChatGPT</AlertTitle>
                  <AlertDescription>
                    Open {runtimeEvent.verificationUri} and enter code <strong>{runtimeEvent.userCode}</strong>.
                  </AlertDescription>
                </Alert>
              )}
              {runtimeEvent?.type === 'auth_url' && (
                <Alert>
                  <ExternalLink />
                  <AlertTitle>Continue in your browser</AlertTitle>
                  <AlertDescription>
                    Fraia opened ChatGPT sign-in in your default browser and will update when authorization finishes.
                  </AlertDescription>
                </Alert>
              )}
              {runtimeEvent?.type === 'progress' && (
                <p className="text-sm text-muted-foreground" role="status">{runtimeEvent.message}</p>
              )}
              {runtimeEvent?.type === 'error' && (
                <Alert variant="destructive">
                  <AlertTitle>ChatGPT sign-in failed</AlertTitle>
                  <AlertDescription>{runtimeEvent.message}</AlertDescription>
                </Alert>
              )}
              {runtimeEvent?.type === 'prompt' && runtimeEvent.flowId && (
                <form onSubmit={(submitEvent) => { submitEvent.preventDefault(); void answerPrompt(runtimeEvent); }}>
                  <FieldGroup>
                    <Field>
                      <FieldContent>
                        <FieldLabel htmlFor="chatgpt-auth-prompt">
                          {runtimeEvent.prompt?.message ?? 'Authentication response'}
                        </FieldLabel>
                        {runtimeEvent.prompt?.type === 'select' ? (
                          <Select
                            value={promptAnswer}
                            items={(runtimeEvent.prompt.options ?? []).map((option) => ({ value: option.id, label: option.label }))}
                            onValueChange={(value) => {
                              if (typeof value === 'string') setPromptAnswer(value);
                            }}
                          >
                            <SelectTrigger id="chatgpt-auth-prompt" className="w-full">
                              <SelectValue placeholder="Choose an option" />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectGroup>
                                {(runtimeEvent.prompt.options ?? []).map((option) => (
                                  <SelectItem key={option.id} value={option.id}>{option.label}</SelectItem>
                                ))}
                              </SelectGroup>
                            </SelectContent>
                          </Select>
                        ) : (
                          <Input
                            id="chatgpt-auth-prompt"
                            type={runtimeEvent.prompt?.type === 'secret' ? 'password' : 'text'}
                            autoComplete="off"
                            value={promptAnswer}
                            onChange={(event) => setPromptAnswer(event.target.value)}
                          />
                        )}
                      </FieldContent>
                      <Button type="submit" disabled={!promptAnswer.trim()}>Continue</Button>
                    </Field>
                  </FieldGroup>
                </form>
              )}
            </>
          )}
        </div>

        <div className="flex items-center justify-between gap-3 text-xs text-muted-foreground">
          <span>{freshness ? `Checked ${new Date(freshness).toLocaleString()}` : 'Waiting for the AI catalogue'}</span>
          <span>Eligible ChatGPT plan required</span>
        </div>

        <DialogFooter className="sm:justify-between">
          <Button type="button" size="sm" variant="ghost" onClick={refresh} disabled={busyAction === 'refresh'}>
            <RefreshCw data-icon="inline-start" className={cn(busyAction === 'refresh' && 'animate-spin')} />
            Refresh
          </Button>
          {connected ? (
            <Button type="button" size="sm" variant="outline" onClick={disconnect} disabled={busyAction === 'disconnect'}>
              {busyAction === 'disconnect'
                ? <LoaderCircle data-icon="inline-start" className="animate-spin" />
                : <Unplug data-icon="inline-start" />}
              Disconnect
            </Button>
          ) : (
            <Button
              type="button"
              size="sm"
              onClick={startOAuth}
              disabled={!secureStorage || !oauth || busyAction === 'sign-in' || authInProgress}
            >
              {busyAction === 'sign-in' || authInProgress
                ? <LoaderCircle data-icon="inline-start" className="animate-spin" />
                : <ExternalLink data-icon="inline-start" />}
              {authInProgress ? 'Waiting for ChatGPT' : 'Sign in with ChatGPT'}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
