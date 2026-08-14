import { useCallback, useEffect, useMemo, useState } from 'react';
import { ExternalLink, RefreshCw, ShieldCheck, Sparkles, Unplug } from 'lucide-react';
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
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemMedia,
  ItemTitle,
} from '@/components/ui/item';
import { Spinner } from '@/components/ui/spinner';
import {
  FRAIA_AI_MODEL_ID,
  FRAIA_AI_PROVIDER_ID,
} from '@/lib/agentOptions';
import type { AiProviderCatalogue } from '@/lib/types';

type RuntimeEvent = {
  kind?: string;
  flowId?: string;
  providerId?: string;
  type?: string;
  message?: string;
  url?: string;
};

function providerState(provider: NonNullable<AiProviderCatalogue['providers']>[number]) {
  return provider.authState ?? provider.auth_state ?? 'disconnected';
}

function shortConnectionError(error: string) {
  if (/timed out/i.test(error)) return 'The connection took too long. Try again.';
  if (/unavailable|failed to contact|could not reach/i.test(error)) return 'Fraia could not reach ChatGPT. Check your connection and try again.';
  return 'Fraia could not finish that action. Try again.';
}

export function AiProvidersDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [catalogue, setCatalogue] = useState<AiProviderCatalogue | null>(null);
  const [busyAction, setBusyAction] = useState<'refresh' | 'sign-in' | 'disconnect' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [runtimeEvent, setRuntimeEvent] = useState<RuntimeEvent | null>(null);

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
              <AlertTitle>{shortConnectionError(error)}</AlertTitle>
              <AlertDescription>
                <details>
                  <summary className="cursor-pointer">Details</summary>
                  <p className="mt-2 break-words">{error}</p>
                </details>
              </AlertDescription>
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
                      Fraia uses one reviewed model for every design conversation.
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
                  Signed in securely. Your authorization stays out of project files.
                </p>
              )}
              {connected && !lunaReady && (
                <Alert variant="destructive">
                  <AlertTitle>Fraia AI is unavailable</AlertTitle>
                  <AlertDescription>
                    Refresh the connection. Fraia will not switch to another model.
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
                  <AlertTitle>{shortConnectionError(runtimeEvent.message ?? '')}</AlertTitle>
                  {runtimeEvent.message && <AlertDescription><details><summary className="cursor-pointer">Details</summary><p className="mt-2 break-words">{runtimeEvent.message}</p></details></AlertDescription>}
                </Alert>
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
            {busyAction === 'refresh'
              ? <Spinner data-icon="inline-start" />
              : <RefreshCw data-icon="inline-start" />}
            Refresh
          </Button>
          {connected ? (
            <Button type="button" size="sm" variant="outline" onClick={disconnect} disabled={busyAction === 'disconnect'}>
              {busyAction === 'disconnect'
                ? <Spinner data-icon="inline-start" />
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
                ? <Spinner data-icon="inline-start" />
                : <ExternalLink data-icon="inline-start" />}
              {authInProgress ? 'Waiting for ChatGPT' : 'Sign in with ChatGPT'}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
