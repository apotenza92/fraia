const crypto = require('node:crypto');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');

const DEFAULT_REFRESH_INTERVAL_MS = 60 * 60 * 1000;
const DEFAULT_FOCUS_DEBOUNCE_MS = 5 * 60 * 1000;
const DEFAULT_STARTUP_TIMEOUT_MS = 8_000;
const DEFAULT_TURN_TIMEOUT_MS = 120_000;
const MAX_REQUEST_BYTES = 8 * 1024 * 1024;
const FRAIA_AI_PROVIDER_ID = 'openai-codex';
const FRAIA_AI_MODEL_ID = 'gpt-5.6-luna';

function publicFraiaCatalogue(catalogue) {
  return {
    ...catalogue,
    providers: (catalogue?.providers ?? []).filter((provider) => provider.id === FRAIA_AI_PROVIDER_ID),
    models: (catalogue?.models ?? []).filter((model) => (
      (model.providerId ?? model.provider_id) === FRAIA_AI_PROVIDER_ID
      && (model.modelId ?? model.model_id ?? model.slug) === FRAIA_AI_MODEL_ID
    )),
  };
}

function atomicWrite(filePath, data) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporaryPath = `${filePath}.${process.pid}.${crypto.randomUUID()}.tmp`;
  fs.writeFileSync(temporaryPath, data, { mode: 0o600 });
  fs.renameSync(temporaryPath, filePath);
}

class SecureCredentialStore {
  constructor({ safeStorage, filePath }) {
    this.safeStorage = safeStorage;
    this.filePath = filePath;
    this.writeChain = Promise.resolve();
  }

  isPersistenceAvailable() {
    return Boolean(this.safeStorage?.isEncryptionAvailable?.());
  }

  readAll() {
    if (!fs.existsSync(this.filePath)) return {};
    if (!this.isPersistenceAvailable()) {
      throw new Error('Secure operating-system credential encryption is unavailable.');
    }
    try {
      const encrypted = fs.readFileSync(this.filePath);
      const plaintext = this.safeStorage.decryptString(encrypted);
      const parsed = JSON.parse(plaintext);
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        throw new Error('credential payload is not an object');
      }
      return parsed;
    } catch (error) {
      throw new Error(`Fraia could not decrypt its AI credential store: ${error?.message ?? error}`);
    }
  }

  writeAll(credentials) {
    if (!this.isPersistenceAvailable()) {
      throw new Error('Secure operating-system credential encryption is unavailable; Fraia will not store credentials as plaintext.');
    }
    const plaintext = JSON.stringify(credentials);
    const encrypted = this.safeStorage.encryptString(plaintext);
    atomicWrite(this.filePath, encrypted);
  }

  async read(providerId) {
    return this.readAll()[providerId];
  }

  async list() {
    return Object.entries(this.readAll()).map(([providerId, credential]) => ({
      providerId,
      type: credential.type,
    }));
  }

  async modify(providerId, fn) {
    let result;
    const operation = this.writeChain.then(async () => {
      const credentials = this.readAll();
      const next = await fn(credentials[providerId]);
      if (next !== undefined) {
        credentials[providerId] = next;
        this.writeAll(credentials);
      }
      result = next ?? credentials[providerId];
    });
    this.writeChain = operation.catch(() => {});
    await operation;
    return result;
  }

  async delete(providerId) {
    const operation = this.writeChain.then(async () => {
      const credentials = this.readAll();
      if (!(providerId in credentials)) return;
      delete credentials[providerId];
      this.writeAll(credentials);
    });
    this.writeChain = operation.catch(() => {});
    await operation;
  }
}

class NonPersistentCredentialStore {
  async read() { return undefined; }
  async list() { return []; }
  async modify() {
    throw new Error('Secure operating-system credential encryption is unavailable; persistent authentication is disabled.');
  }
  async delete() {}
}

function reasoningLevels(model) {
  if (!model?.reasoning) return [{ effort: 'off', description: 'Reasoning is not exposed for this model.' }];
  // Pi's thinkingLevelMap translates public levels for a provider; its keys are
  // not an availability list. Pi accepts this complete set and clamps each
  // level to the selected model's actual transport capability.
  const levels = ['off', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'];
  return levels.map((effort) => ({ effort, description: `${effort[0].toUpperCase()}${effort.slice(1)} reasoning` }));
}

function typeBoxSchema(Type, schema) {
  if (!schema || typeof schema !== 'object') return Type.Unknown();
  if (Array.isArray(schema.enum)) {
    return Type.Union(schema.enum.map((value) => (value === null ? Type.Null() : Type.Literal(value))));
  }
  if (Array.isArray(schema.type)) {
    return Type.Union(schema.type.map((type) => typeBoxSchema(Type, { ...schema, type })));
  }
  if (Array.isArray(schema.anyOf)) return Type.Union(schema.anyOf.map((item) => typeBoxSchema(Type, item)));
  if (Array.isArray(schema.oneOf)) return Type.Union(schema.oneOf.map((item) => typeBoxSchema(Type, item)));
  if (schema.type === 'array') {
    return Type.Array(typeBoxSchema(Type, schema.items ?? {}), {
      ...(Number.isFinite(schema.minItems) ? { minItems: schema.minItems } : {}),
      ...(Number.isFinite(schema.maxItems) ? { maxItems: schema.maxItems } : {}),
    });
  }
  if (schema.type === 'object' || schema.properties) {
    const required = new Set(schema.required ?? []);
    const properties = Object.fromEntries(Object.entries(schema.properties ?? {}).map(([key, value]) => {
      const converted = typeBoxSchema(Type, value);
      return [key, required.has(key) ? converted : Type.Optional(converted)];
    }));
    return Type.Object(properties, { additionalProperties: schema.additionalProperties === true });
  }
  if (schema.type === 'string') return Type.String();
  if (schema.type === 'integer') return Type.Integer();
  if (schema.type === 'number') return Type.Number();
  if (schema.type === 'boolean') return Type.Boolean();
  if (schema.type === 'null') return Type.Null();
  return Type.Unknown();
}

function normalizeError(error) {
  if (error?.name === 'AbortError') return 'The AI turn was cancelled.';
  return error?.message ?? String(error);
}

class FraiaAiRuntime {
  constructor({
    safeStorage,
    shell,
    userDataDir,
    importPi = () => import('@earendil-works/pi-coding-agent'),
    importTypeBox = () => import('typebox'),
    emitStatus = () => {},
    refreshIntervalMs = DEFAULT_REFRESH_INTERVAL_MS,
    focusDebounceMs = DEFAULT_FOCUS_DEBOUNCE_MS,
    startupTimeoutMs = DEFAULT_STARTUP_TIMEOUT_MS,
    turnTimeoutMs = DEFAULT_TURN_TIMEOUT_MS,
  }) {
    this.safeStorage = safeStorage;
    this.shell = shell;
    this.userDataDir = userDataDir;
    this.importPi = importPi;
    this.importTypeBox = importTypeBox;
    this.emitStatus = emitStatus;
    this.refreshIntervalMs = refreshIntervalMs;
    this.focusDebounceMs = focusDebounceMs;
    this.startupTimeoutMs = startupTimeoutMs;
    this.turnTimeoutMs = turnTimeoutMs;
    this.modelRuntime = null;
    this.pi = null;
    this.Type = null;
    this.server = null;
    this.serverToken = null;
    this.serverUrl = null;
    this.refreshTimer = null;
    this.lastFocusRefreshAt = 0;
    this.catalogRefreshedAt = null;
    this.catalogRefreshError = null;
    this.activeTurns = new Map();
    this.authFlows = new Map();
  }

  async initialize() {
    const [pi, typebox] = await Promise.all([this.importPi(), this.importTypeBox()]);
    this.pi = pi;
    this.Type = typebox.Type;
    const credentialFile = path.join(this.userDataDir, 'ai', 'credentials.bin');
    this.credentials = this.safeStorage?.isEncryptionAvailable?.()
      ? new SecureCredentialStore({ safeStorage: this.safeStorage, filePath: credentialFile })
      : new NonPersistentCredentialStore();
    this.modelRuntime = await pi.ModelRuntime.create({
      credentials: this.credentials,
      modelsPath: path.join(this.userDataDir, 'ai', 'provider-config.json'),
      modelsStorePath: path.join(this.userDataDir, 'ai', 'models-cache.json'),
      allowModelNetwork: true,
      modelRefreshTimeoutMs: this.startupTimeoutMs,
    });
    this.catalogRefreshedAt = new Date().toISOString();
    this.catalogRefreshError = this.modelRuntime.getError?.() ?? null;
    this.refreshTimer = setInterval(() => {
      void this.refreshCatalog('periodic');
    }, this.refreshIntervalMs);
    this.refreshTimer.unref?.();
    return this;
  }

  persistenceAvailable() {
    return this.credentials instanceof SecureCredentialStore && this.credentials.isPersistenceAvailable();
  }

  async catalog() {
    if (!this.modelRuntime) throw new Error('Fraia AI runtime is not initialized.');
    const available = await this.modelRuntime.getAvailable();
    const availableIds = new Set(available.map((model) => `${model.provider}/${model.id}`));
    const providers = await Promise.all(this.modelRuntime.getProviders().map(async (provider) => {
      const status = this.modelRuntime.getProviderAuthStatus(provider.id);
      const check = await this.modelRuntime.checkAuth(provider.id).catch(() => undefined);
      const authentication = [];
      if (provider.auth?.oauth) {
        authentication.push({
          type: 'oauth',
          label: provider.auth.oauth.loginLabel ?? provider.auth.oauth.name,
          interactive: true,
          persistentAllowed: this.persistenceAvailable(),
        });
      }
      if (provider.auth?.apiKey) {
        authentication.push({
          type: provider.auth.apiKey.login ? 'api_key' : 'external',
          label: provider.auth.apiKey.name,
          interactive: Boolean(provider.auth.apiKey.login),
          persistentAllowed: provider.auth.apiKey.login ? this.persistenceAvailable() : false,
          requirements: status.label ? [status.label] : [],
        });
      }
      return {
        id: provider.id,
        name: provider.name,
        authentication,
        authState: check ? 'connected' : (status.configured ? 'configured' : 'disconnected'),
        authType: check?.type ?? null,
        authSource: check?.source ?? status.label ?? status.source ?? null,
      };
    }));
    const models = this.modelRuntime.getModels().map((model) => ({
      providerId: model.provider,
      modelId: model.id,
      displayName: model.name || model.id,
      available: availableIds.has(`${model.provider}/${model.id}`),
      reasoning: Boolean(model.reasoning),
      defaultReasoningLevel: model.reasoning ? 'low' : 'off',
      supportedReasoningLevels: reasoningLevels(model),
      contextWindow: model.contextWindow ?? null,
      maxTokens: model.maxTokens ?? null,
    }));
    return {
      providers,
      models,
      catalogue: {
        refreshedAt: this.catalogRefreshedAt,
        stale: Boolean(this.catalogRefreshError),
        refreshError: this.catalogRefreshError,
        source: models.length ? 'pi-runtime' : 'unavailable',
      },
      secureCredentialStorageAvailable: this.persistenceAvailable(),
    };
  }

  async refreshCatalog(reason = 'manual') {
    try {
      await this.modelRuntime.refresh({ reason });
      this.catalogRefreshedAt = new Date().toISOString();
      this.catalogRefreshError = null;
      this.emitStatus({ kind: 'catalogue', state: 'refreshed', reason, at: this.catalogRefreshedAt });
    } catch (error) {
      this.catalogRefreshError = normalizeError(error);
      this.emitStatus({ kind: 'catalogue', state: 'stale', reason, message: this.catalogRefreshError });
    }
    return this.catalog();
  }

  async refreshAfterFocus() {
    const now = Date.now();
    if (now - this.lastFocusRefreshAt < this.focusDebounceMs) return this.catalog();
    this.lastFocusRefreshAt = now;
    return this.refreshCatalog('focus');
  }

  async submitApiKey(providerId, apiKey) {
    if (!this.persistenceAvailable()) {
      throw new Error('Secure operating-system credential encryption is unavailable; Fraia will not persist an API key.');
    }
    const secret = String(apiKey ?? '');
    if (!secret.trim()) throw new Error('Enter an API key.');
    await this.modelRuntime.login(providerId, 'api_key', {
      prompt: async (prompt) => {
        if (prompt.type !== 'secret') throw new Error(`Provider requested unsupported ${prompt.type} input during API-key setup.`);
        return secret;
      },
      notify: () => {},
    });
    await this.refreshCatalog('authentication');
    return this.catalog();
  }

  async startOAuth(providerId) {
    if (!this.persistenceAvailable()) {
      throw new Error('Secure operating-system credential encryption is unavailable; Fraia cannot persist an OAuth connection.');
    }
    const flowId = crypto.randomUUID();
    const abortController = new AbortController();
    const pendingPrompts = [];
    this.authFlows.set(flowId, { providerId, abortController, pendingPrompts });
    const emit = (event) => this.emitStatus({ kind: 'authentication', flowId, providerId, ...event });
    void this.modelRuntime.login(providerId, 'oauth', {
      signal: abortController.signal,
      notify: (event) => {
        emit(event);
        const url = event.type === 'auth_url' ? event.url : event.type === 'device_code' ? event.verificationUri : null;
        if (url) void this.shell?.openExternal?.(url);
      },
      prompt: (prompt) => new Promise((resolve, reject) => {
        const entry = { resolve, reject };
        pendingPrompts.push(entry);
        emit({ type: 'prompt', prompt: { ...prompt, signal: undefined } });
        prompt.signal?.addEventListener('abort', () => reject(new Error('Authentication prompt was cancelled.')), { once: true });
      }),
    }).then(async () => {
      emit({ type: 'complete' });
      await this.refreshCatalog('authentication');
    }).catch((error) => {
      emit({ type: 'error', message: normalizeError(error) });
    }).finally(() => {
      this.authFlows.delete(flowId);
    });
    return { flowId };
  }

  answerAuthPrompt(flowId, value) {
    const flow = this.authFlows.get(flowId);
    const prompt = flow?.pendingPrompts.shift();
    if (!prompt) throw new Error('No authentication prompt is awaiting a response.');
    prompt.resolve(String(value ?? ''));
    return { ok: true };
  }

  cancelAuth(flowId) {
    const flow = this.authFlows.get(flowId);
    if (!flow) return { ok: false };
    flow.abortController.abort();
    for (const prompt of flow.pendingPrompts) prompt.reject(new Error('Authentication cancelled.'));
    this.authFlows.delete(flowId);
    return { ok: true };
  }

  async disconnect(providerId) {
    await this.modelRuntime.logout(providerId);
    await this.refreshCatalog('authentication');
    return this.catalog();
  }

  async runTurn(request) {
    const { requestId, providerId, modelId, reasoningEffort, prompt, responseSchema } = request;
    if (!requestId || !providerId || !modelId || !prompt || !responseSchema) {
      throw new Error('AI turn request is missing required fields.');
    }
    if (this.activeTurns.has(requestId)) throw new Error(`AI request ${requestId} is already active.`);
    const model = this.modelRuntime.getModel(providerId, modelId);
    if (!model) throw new Error(`Pi does not know model ${providerId}/${modelId}.`);
    const available = await this.modelRuntime.getAvailable(providerId);
    if (!available.some((candidate) => candidate.id === modelId)) {
      throw new Error(`Selected model ${providerId}/${modelId} is unavailable. Choose an available authenticated model.`);
    }

    let structuredResult = null;
    const schema = typeBoxSchema(this.Type, responseSchema);
    const tool = this.pi.defineTool({
      name: 'submit_fraia_response',
      label: 'Submit Fraia response',
      description: 'Submit the final structured response to Fraia. This must be the final action.',
      promptSnippet: 'Submit a validated Fraia response',
      promptGuidelines: [
        'Always finish by calling submit_fraia_response exactly once.',
        'Do not provide the final result as unstructured assistant text.',
      ],
      parameters: schema,
      async execute(_toolCallId, params) {
        structuredResult = params;
        return {
          content: [{ type: 'text', text: 'Fraia received the structured response.' }],
          details: { accepted: true },
          terminate: true,
        };
      },
    });
    const settingsManager = this.pi.SettingsManager.inMemory({
      compaction: { enabled: false },
      retry: { enabled: false },
      enableAnalytics: false,
      enableSkillCommands: false,
    });
    const cwd = path.join(os.tmpdir(), 'fraia-pi-runtime');
    fs.mkdirSync(cwd, { recursive: true });
    const resourceLoader = new this.pi.DefaultResourceLoader({
      cwd,
      agentDir: cwd,
      settingsManager,
      noExtensions: true,
      noSkills: true,
      noPromptTemplates: true,
      noThemes: true,
      noContextFiles: true,
      systemPrompt: 'You are Fraia\'s constrained AI reasoning adapter. Use only the supplied prompt. Do not infer access to files, tools, repositories, or project context. Finish by calling submit_fraia_response with arguments matching its schema.',
    });
    await resourceLoader.reload();
    const { session } = await this.pi.createAgentSession({
      cwd,
      modelRuntime: this.modelRuntime,
      model,
      thinkingLevel: reasoningEffort === 'off' ? 'minimal' : reasoningEffort,
      noTools: 'all',
      tools: ['submit_fraia_response'],
      customTools: [tool],
      resourceLoader,
      sessionManager: this.pi.SessionManager.inMemory(cwd),
      settingsManager,
    });
    const timeout = setTimeout(() => void session.abort(), this.turnTimeoutMs);
    this.activeTurns.set(requestId, session);
    this.emitStatus({ kind: 'turn', requestId, state: 'generating', providerId, modelId });
    try {
      await session.prompt(prompt, { expandPromptTemplates: false });
      if (!structuredResult) {
        this.emitStatus({ kind: 'turn', requestId, state: 'correcting' });
        await session.prompt('Your previous response did not call submit_fraia_response with valid arguments. Call it now with a complete response matching the tool schema.', { expandPromptTemplates: false });
      }
      if (!structuredResult) {
        throw new Error('The model did not submit a valid structured Fraia response after one corrective attempt.');
      }
      this.emitStatus({ kind: 'turn', requestId, state: 'validating' });
      return {
        output: structuredResult,
        providerId,
        modelId,
        reasoningEffort,
        catalogueRefreshedAt: this.catalogRefreshedAt,
      };
    } finally {
      clearTimeout(timeout);
      this.activeTurns.delete(requestId);
      session.dispose();
    }
  }

  async cancelTurn(requestId) {
    const session = this.activeTurns.get(requestId);
    if (!session) return false;
    await session.abort();
    this.emitStatus({ kind: 'turn', requestId, state: 'cancelled' });
    return true;
  }

  async startLoopback() {
    if (this.server) return { url: this.serverUrl, token: this.serverToken };
    this.serverToken = crypto.randomBytes(32).toString('base64url');
    this.server = http.createServer(async (request, response) => {
      try {
        if (request.headers.authorization !== `Bearer ${this.serverToken}`) {
          response.writeHead(401, { 'content-type': 'application/json' });
          response.end(JSON.stringify({ error: 'unauthorized' }));
          return;
        }
        const url = new URL(request.url, 'http://127.0.0.1');
        if (request.method === 'GET' && url.pathname === '/v1/catalog') {
          return this.sendJson(response, 200, await this.catalog());
        }
        if (request.method === 'POST' && url.pathname === '/v1/turns') {
          const body = await this.readJson(request);
          return this.sendJson(response, 200, await this.runTurn(body));
        }
        if (request.method === 'DELETE' && url.pathname.startsWith('/v1/turns/')) {
          const requestId = decodeURIComponent(url.pathname.slice('/v1/turns/'.length));
          return this.sendJson(response, 200, { cancelled: await this.cancelTurn(requestId) });
        }
        return this.sendJson(response, 404, { error: 'not found' });
      } catch (error) {
        return this.sendJson(response, 400, { error: normalizeError(error) });
      }
    });
    await new Promise((resolve, reject) => {
      this.server.once('error', reject);
      this.server.listen(0, '127.0.0.1', resolve);
    });
    const address = this.server.address();
    this.serverUrl = `http://127.0.0.1:${address.port}`;
    return { url: this.serverUrl, token: this.serverToken };
  }

  readJson(request) {
    return new Promise((resolve, reject) => {
      const chunks = [];
      let size = 0;
      request.on('data', (chunk) => {
        size += chunk.length;
        if (size > MAX_REQUEST_BYTES) {
          reject(new Error('AI runtime request is too large.'));
          request.destroy();
          return;
        }
        chunks.push(chunk);
      });
      request.on('end', () => {
        try { resolve(JSON.parse(Buffer.concat(chunks).toString('utf8'))); } catch (error) { reject(error); }
      });
      request.on('error', reject);
    });
  }

  sendJson(response, status, body) {
    response.writeHead(status, { 'content-type': 'application/json', 'cache-control': 'no-store' });
    response.end(JSON.stringify(body));
  }

  async stop() {
    if (this.refreshTimer) clearInterval(this.refreshTimer);
    for (const requestId of [...this.activeTurns.keys()]) await this.cancelTurn(requestId);
    for (const flowId of [...this.authFlows.keys()]) this.cancelAuth(flowId);
    if (this.server) await new Promise((resolve) => this.server.close(resolve));
    this.server = null;
  }
}

function fakeValueForSchema(schema) {
  if (!schema || typeof schema !== 'object') return null;
  if (Array.isArray(schema.enum)) return schema.enum.find((value) => value !== null) ?? null;
  if (Array.isArray(schema.type)) {
    const preferred = schema.type.find((type) => type !== 'null') ?? 'null';
    return fakeValueForSchema({ ...schema, type: preferred });
  }
  if (schema.type === 'object' || schema.properties) {
    return Object.fromEntries((schema.required ?? []).map((key) => [key, fakeValueForSchema(schema.properties?.[key] ?? {})]));
  }
  if (schema.type === 'array') return [];
  if (schema.type === 'string') return 'Fake Pi response';
  if (schema.type === 'integer' || schema.type === 'number') return 1;
  if (schema.type === 'boolean') return false;
  return null;
}

class FakeFraiaAiRuntime extends FraiaAiRuntime {
  async initialize() {
    const credentialFile = path.join(this.userDataDir, 'ai', 'credentials.bin');
    this.credentials = this.safeStorage?.isEncryptionAvailable?.()
      ? new SecureCredentialStore({ safeStorage: this.safeStorage, filePath: credentialFile })
      : new NonPersistentCredentialStore();
    this.catalogRefreshedAt = new Date().toISOString();
    return this;
  }

  async isConnected() {
    return Boolean(await this.credentials.read('openai-codex'));
  }

  async catalog() {
    const connected = await this.isConnected();
    return {
      providers: [{
        id: 'openai-codex',
        name: 'OpenAI Codex',
        authentication: [{ type: 'oauth', label: 'Sign in with ChatGPT', interactive: true, persistentAllowed: this.persistenceAvailable() }],
        authState: connected ? 'connected' : 'disconnected',
        authType: connected ? 'oauth' : null,
        authSource: connected ? 'Encrypted ChatGPT test authorization' : null,
      }],
      models: [{
        providerId: 'openai-codex',
        modelId: 'gpt-5.6-luna',
        displayName: 'GPT-5.6 Luna',
        available: connected,
        reasoning: true,
        defaultReasoningLevel: 'low',
        supportedReasoningLevels: reasoningLevels({ reasoning: true }),
        contextWindow: 128000,
        maxTokens: 16384,
      }],
      catalogue: { refreshedAt: this.catalogRefreshedAt, stale: false, refreshError: null, source: 'fake-pi-runtime' },
      secureCredentialStorageAvailable: this.persistenceAvailable(),
    };
  }

  async refreshCatalog(reason = 'manual') {
    this.catalogRefreshedAt = new Date().toISOString();
    this.emitStatus({ kind: 'catalogue', state: 'refreshed', reason, at: this.catalogRefreshedAt });
    return this.catalog();
  }

  async startOAuth(providerId) {
    if (providerId !== 'openai-codex') throw new Error('Unknown fake provider.');
    if (!this.persistenceAvailable()) throw new Error('Secure operating-system credential encryption is unavailable.');
    const flowId = crypto.randomUUID();
    this.emitStatus({
      kind: 'authentication',
      flowId,
      providerId,
      type: 'progress',
      message: 'Completing fake ChatGPT sign-in.',
    });
    await this.credentials.modify(providerId, async () => ({
      type: 'oauth',
      access: 'fake-chatgpt-access-token',
      refresh: 'fake-chatgpt-refresh-token',
      expires: Date.now() + 60 * 60 * 1000,
    }));
    await this.refreshCatalog('authentication');
    this.emitStatus({ kind: 'authentication', flowId, providerId, type: 'complete' });
    return { flowId };
  }

  async disconnect(providerId) {
    await this.credentials.delete(providerId);
    return this.refreshCatalog('authentication');
  }

  async runTurn(request) {
    if (!(await this.isConnected())) throw new Error('The fake provider is disconnected.');
    if (request.providerId !== 'openai-codex' || request.modelId !== 'gpt-5.6-luna') {
      throw new Error('The selected fake model is unavailable.');
    }
    const delayMs = Number.parseInt(process.env.FRAIA_FAKE_AI_TURN_DELAY_MS || '0', 10);
    try {
      if (delayMs > 0) {
        await new Promise((resolve, reject) => {
          const timer = setTimeout(resolve, delayMs);
          this.activeTurns.set(request.requestId, {
            abort: async () => {
              clearTimeout(timer);
              reject(new Error('The AI turn was cancelled.'));
            },
          });
        });
      }
      return {
        output: fakeValueForSchema(request.responseSchema),
        providerId: request.providerId,
        modelId: request.modelId,
        reasoningEffort: request.reasoningEffort,
        catalogueRefreshedAt: this.catalogRefreshedAt,
      };
    } finally {
      this.activeTurns.delete(request.requestId);
    }
  }
}

module.exports = {
  FakeFraiaAiRuntime,
  FraiaAiRuntime,
  NonPersistentCredentialStore,
  SecureCredentialStore,
  publicFraiaCatalogue,
  reasoningLevels,
  typeBoxSchema,
  fakeValueForSchema,
};
