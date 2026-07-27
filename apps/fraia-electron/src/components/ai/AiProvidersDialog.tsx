import { useCallback, useEffect, useMemo, useState } from 'react';
import { ExternalLink, KeyRound, LoaderCircle, RefreshCw, Unplug, Waypoints } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty';
import { Field, FieldContent, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemTitle } from '@/components/ui/item';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { cn } from '@/lib/utils';
import type { AgentProviderDescriptor, AiProviderCatalogue } from '@/lib/types';

type RuntimeEvent = {
  kind?: string;
  flowId?: string;
  providerId?: string;
  type?: string;
  message?: string;
  userCode?: string;
  verificationUri?: string;
  prompt?: { type?: string; message?: string; options?: Array<{ id: string; label: string }> };
};

function providerState(provider: AgentProviderDescriptor) {
  return provider.authState ?? provider.auth_state ?? 'disconnected';
}

export function AiProvidersDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [catalogue, setCatalogue] = useState<AiProviderCatalogue | null>(null);
  const [busyProvider, setBusyProvider] = useState<string | null>(null);
  const [apiKeys, setApiKeys] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const [runtimeEvents, setRuntimeEvents] = useState<Record<string, RuntimeEvent>>({});
  const [promptAnswers, setPromptAnswers] = useState<Record<string, string>>({});
  const [providerQuery, setProviderQuery] = useState('');

  const load = useCallback(async () => {
    setError(null);
    try {
      setCatalogue(await window.fraia.aiProviders());
    } catch (cause: any) {
      setError(cause?.message || 'Could not load AI providers.');
    }
  }, []);

  useEffect(() => {
    if (open) void load();
  }, [load, open]);

  useEffect(() => {
    const unsubscribe: unknown = window.fraia.onAiRuntimeStatus?.((event: RuntimeEvent) => {
      if (event.kind !== 'authentication' || !event.providerId) return;
      setRuntimeEvents((current) => ({ ...current, [event.providerId!]: event }));
      if (event.type === 'complete') void load();
    });
    return () => {
      if (typeof unsubscribe === 'function') unsubscribe();
    };
  }, [load]);

  const providerModels = useMemo(() => {
    const counts = new Map<string, { total: number; available: number }>();
    for (const model of catalogue?.models ?? []) {
      const providerId = model.providerId ?? model.provider_id ?? '';
      const count = counts.get(providerId) ?? { total: 0, available: 0 };
      count.total += 1;
      if (model.available !== false) count.available += 1;
      counts.set(providerId, count);
    }
    return counts;
  }, [catalogue]);

  const visibleProviders = useMemo(() => {
    const query = providerQuery.trim().toLocaleLowerCase();
    if (!query) return catalogue?.providers ?? [];
    return (catalogue?.providers ?? []).filter((provider) => (
      provider.name.toLocaleLowerCase().includes(query)
      || provider.id.toLocaleLowerCase().includes(query)
      || provider.authentication.some((method) => method.label.toLocaleLowerCase().includes(query))
    ));
  }, [catalogue, providerQuery]);

  async function submitApiKey(providerId: string) {
    const apiKey = apiKeys[providerId] ?? '';
    setBusyProvider(providerId);
    setError(null);
    try {
      setCatalogue(await window.fraia.aiSubmitApiKey({ providerId, apiKey }));
      setApiKeys((current) => ({ ...current, [providerId]: '' }));
    } catch (cause: any) {
      setError(cause?.message || 'Could not save the API key.');
    } finally {
      setApiKeys((current) => ({ ...current, [providerId]: '' }));
      setBusyProvider(null);
    }
  }

  async function startOAuth(providerId: string) {
    setBusyProvider(providerId);
    setError(null);
    try {
      await window.fraia.aiStartOAuth({ providerId });
    } catch (cause: any) {
      setError(cause?.message || 'Could not start provider sign-in.');
    } finally {
      setBusyProvider(null);
    }
  }

  async function disconnect(providerId: string) {
    setBusyProvider(providerId);
    setError(null);
    try {
      setCatalogue(await window.fraia.aiDisconnect({ providerId }));
    } catch (cause: any) {
      setError(cause?.message || 'Could not disconnect the provider.');
    } finally {
      setBusyProvider(null);
    }
  }

  async function refresh() {
    setBusyProvider('catalogue');
    setError(null);
    try {
      setCatalogue(await window.fraia.aiRefreshCatalog());
    } catch (cause: any) {
      setError(cause?.message || 'Could not refresh the model catalogue.');
    } finally {
      setBusyProvider(null);
    }
  }

  async function answerPrompt(providerId: string, event: RuntimeEvent) {
    if (!event.flowId) return;
    await window.fraia.aiAnswerAuthPrompt({ flowId: event.flowId, value: promptAnswers[providerId] ?? '' });
    setPromptAnswers((current) => ({ ...current, [providerId]: '' }));
  }

  const secureStorage = catalogue?.secureCredentialStorageAvailable ?? catalogue?.secure_credential_storage_available;
  const freshness = catalogue?.catalogue?.refreshedAt ?? catalogue?.catalogue?.refreshed_at;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[min(46rem,calc(100vh-2rem))] sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>AI providers</DialogTitle>
          <DialogDescription>Connect providers here. Fraia stores credentials with operating-system encryption and keeps them out of projects.</DialogDescription>
        </DialogHeader>
        <div className="flex items-center justify-between gap-3">
          <p className="text-xs text-muted-foreground">{freshness ? `Catalogue refreshed ${new Date(freshness).toLocaleString()}` : 'Catalogue freshness unavailable'}</p>
          <Button type="button" size="sm" variant="outline" onClick={refresh} disabled={busyProvider === 'catalogue'}>
            <RefreshCw data-icon="inline-start" className={cn(busyProvider === 'catalogue' && 'animate-spin')} />
            Refresh
          </Button>
        </div>
        <Field>
          <FieldLabel className="sr-only" htmlFor="provider-search">Search providers</FieldLabel>
          <Input
            id="provider-search"
            type="search"
            autoComplete="off"
            placeholder="Search providers"
            value={providerQuery}
            onChange={(event) => setProviderQuery(event.target.value)}
          />
        </Field>
        {!secureStorage && (
          <Alert variant="destructive">
            <AlertDescription>Secure operating-system encryption is unavailable. Fraia will not accept persistent API keys or OAuth credentials; environment-managed providers may still work.</AlertDescription>
          </Alert>
        )}
        {error && <Alert variant="destructive"><AlertDescription>{error}</AlertDescription></Alert>}
        <ScrollArea className="min-h-0 max-h-[56vh] pr-3">
          <ItemGroup>
            {visibleProviders.map((provider) => {
              const state = providerState(provider);
              const counts = providerModels.get(provider.id) ?? { total: 0, available: 0 };
              const event = runtimeEvents[provider.id];
              const apiKey = provider.authentication.find((method) => method.type === 'api_key');
              const oauth = provider.authentication.find((method) => method.type === 'oauth');
              const external = provider.authentication.filter((method) => method.type === 'external');
              return (
                <Item key={provider.id} role="listitem" size="sm">
                  <ItemHeader>
                    <ItemContent>
                      <ItemTitle>{provider.name}</ItemTitle>
                      <ItemDescription>{counts.available} available of {counts.total} known models</ItemDescription>
                    </ItemContent>
                    <ItemActions>
                      <Badge variant={state === 'connected' || state === 'configured' ? 'secondary' : 'outline'}>{state}</Badge>
                    </ItemActions>
                  </ItemHeader>
                  <ItemContent className="basis-full gap-3">
                    {(state === 'connected' || state === 'configured') && (
                      <ItemFooter>
                        <ItemDescription>{provider.authSource ?? provider.auth_source ?? 'Provider authentication is available.'}</ItemDescription>
                        <Button type="button" size="sm" variant="outline" onClick={() => disconnect(provider.id)} disabled={busyProvider === provider.id}>
                          <Unplug data-icon="inline-start" />
                          Disconnect
                        </Button>
                      </ItemFooter>
                    )}
                    {oauth && state === 'disconnected' && (
                      <Button type="button" size="sm" variant="outline" onClick={() => startOAuth(provider.id)} disabled={!secureStorage || busyProvider === provider.id}>
                        {busyProvider === provider.id
                          ? <LoaderCircle data-icon="inline-start" className="animate-spin" />
                          : <ExternalLink data-icon="inline-start" />}
                        {oauth.label}
                      </Button>
                    )}
                    {apiKey && state === 'disconnected' && (
                      <form onSubmit={(event) => { event.preventDefault(); void submitApiKey(provider.id); }}>
                        <FieldGroup className="gap-3">
                          <Field orientation="responsive" data-disabled={!secureStorage || undefined}>
                            <FieldContent>
                              <FieldLabel htmlFor={`provider-key-${provider.id}`}>{apiKey.label}</FieldLabel>
                              <Input id={`provider-key-${provider.id}`} type="password" autoComplete="off" value={apiKeys[provider.id] ?? ''} disabled={!secureStorage} onChange={(event) => setApiKeys((current) => ({ ...current, [provider.id]: event.target.value }))} />
                            </FieldContent>
                            <Button type="submit" disabled={!secureStorage || !apiKeys[provider.id]?.trim() || busyProvider === provider.id}>
                              <KeyRound data-icon="inline-start" />
                              Connect
                            </Button>
                          </Field>
                        </FieldGroup>
                      </form>
                    )}
                    {external.map((method) => (
                      <ItemDescription key={method.label}>
                        {method.label}{method.requirements?.length ? `: ${method.requirements.join(', ')}` : ' is configured outside Fraia.'}
                      </ItemDescription>
                    ))}
                    {event?.type === 'device_code' && (
                      <Alert><AlertDescription>Open {event.verificationUri} and enter code <strong>{event.userCode}</strong>.</AlertDescription></Alert>
                    )}
                    {event?.type === 'progress' && <ItemDescription role="status">{event.message}</ItemDescription>}
                    {event?.type === 'error' && <Alert variant="destructive"><AlertDescription>{event.message}</AlertDescription></Alert>}
                    {event?.type === 'prompt' && event.flowId && (
                      <form onSubmit={(submitEvent) => { submitEvent.preventDefault(); void answerPrompt(provider.id, event); }}>
                        <FieldGroup className="gap-3">
                          <Field orientation="responsive">
                            <FieldContent>
                              <FieldLabel htmlFor={`provider-prompt-${provider.id}`}>{event.prompt?.message ?? 'Authentication response'}</FieldLabel>
                              {event.prompt?.type === 'select' ? (
                                <Select
                                  value={promptAnswers[provider.id] ?? ''}
                                  items={(event.prompt.options ?? []).map((option) => ({ value: option.id, label: option.label }))}
                                  onValueChange={(value) => {
                                    if (typeof value === 'string') setPromptAnswers((current) => ({ ...current, [provider.id]: value }));
                                  }}
                                >
                                  <SelectTrigger id={`provider-prompt-${provider.id}`} className="w-full"><SelectValue placeholder="Choose an option" /></SelectTrigger>
                                  <SelectContent><SelectGroup>{(event.prompt.options ?? []).map((option) => <SelectItem key={option.id} value={option.id}>{option.label}</SelectItem>)}</SelectGroup></SelectContent>
                                </Select>
                              ) : (
                                <Input id={`provider-prompt-${provider.id}`} type={event.prompt?.type === 'secret' ? 'password' : 'text'} autoComplete="off" value={promptAnswers[provider.id] ?? ''} onChange={(changeEvent) => setPromptAnswers((current) => ({ ...current, [provider.id]: changeEvent.target.value }))} />
                              )}
                            </FieldContent>
                            <Button type="submit" disabled={!promptAnswers[provider.id]?.trim()}>Continue</Button>
                          </Field>
                        </FieldGroup>
                      </form>
                    )}
                  </ItemContent>
                </Item>
              );
            })}
            {catalogue && !visibleProviders.length && (
              <Empty>
                <EmptyHeader>
                  <EmptyMedia variant="icon"><Waypoints /></EmptyMedia>
                  <EmptyTitle>{providerQuery.trim() ? 'No matching providers' : 'No providers available'}</EmptyTitle>
                  <EmptyDescription>
                    {providerQuery.trim()
                      ? `No providers match “${providerQuery.trim()}”.`
                      : 'Fraia could not find any Pi providers in the current catalogue.'}
                  </EmptyDescription>
                </EmptyHeader>
              </Empty>
            )}
          </ItemGroup>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  );
}
