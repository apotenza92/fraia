const crypto = require('node:crypto');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');

const DEFAULT_REFRESH_INTERVAL_MS = 60 * 60 * 1000;
const DEFAULT_FOCUS_DEBOUNCE_MS = 5 * 60 * 1000;
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

function fakeAiTestSafeStorage() {
  const key = crypto.createHash('sha256')
    .update('Fraia fake AI runtime credentials; test tokens only')
    .digest();
  const prefix = Buffer.from('fraia-fake-ai-v1\0');
  return {
    isEncryptionAvailable: () => true,
    encryptString: (value) => {
      const nonce = crypto.randomBytes(12);
      const cipher = crypto.createCipheriv('aes-256-gcm', key, nonce);
      const ciphertext = Buffer.concat([cipher.update(value, 'utf8'), cipher.final()]);
      return Buffer.concat([prefix, nonce, cipher.getAuthTag(), ciphertext]);
    },
    decryptString: (value) => {
      if (!Buffer.isBuffer(value) || value.length < prefix.length + 28 || !value.subarray(0, prefix.length).equals(prefix)) {
        throw new Error('invalid fake AI credential ciphertext');
      }
      const nonce = value.subarray(prefix.length, prefix.length + 12);
      const authenticationTag = value.subarray(prefix.length + 12, prefix.length + 28);
      const ciphertext = value.subarray(prefix.length + 28);
      const decipher = crypto.createDecipheriv('aes-256-gcm', key, nonce);
      decipher.setAuthTag(authenticationTag);
      return Buffer.concat([decipher.update(ciphertext), decipher.final()]).toString('utf8');
    },
  };
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

function refreshErrorMessage(result) {
  const errors = [...(result?.errors?.entries?.() ?? [])];
  if (!errors.length) return null;
  return errors
    .map(([providerId, error]) => `${providerId}: ${normalizeError(error)}`)
    .join('; ');
}

async function importReviewedPiRuntime() {
  const [piAi, piAgent, openAiCodex] = await Promise.all([
    import('@earendil-works/pi-ai'),
    import('@earendil-works/pi-agent-core'),
    import('@earendil-works/pi-ai/providers/openai-codex'),
  ]);
  return {
    Agent: piAgent.Agent,
    createModels: piAi.createModels,
    openaiCodexProvider: openAiCodex.openaiCodexProvider,
  };
}

class FraiaAiRuntime {
  constructor({
    safeStorage,
    shell,
    userDataDir,
    importPi = importReviewedPiRuntime,
    importTypeBox = () => import('typebox'),
    emitStatus = () => {},
    refreshIntervalMs = DEFAULT_REFRESH_INTERVAL_MS,
    focusDebounceMs = DEFAULT_FOCUS_DEBOUNCE_MS,
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
    this.modelRuntime = pi.createModels({
      credentials: this.credentials,
    });
    this.modelRuntime.setProvider(pi.openaiCodexProvider());
    const refreshResult = await this.modelRuntime.refresh({ allowNetwork: false });
    this.catalogRefreshedAt = new Date().toISOString();
    this.catalogRefreshError = refreshErrorMessage(refreshResult);
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
    const providers = await Promise.all(this.modelRuntime.getProviders()
      .filter((provider) => provider.id === FRAIA_AI_PROVIDER_ID)
      .map(async (provider) => {
      const check = await this.modelRuntime.checkAuth(provider.id).catch(() => undefined);
      const stored = await this.credentials.read(provider.id).catch(() => undefined);
      const authentication = provider.auth?.oauth
        ? [{
          type: 'oauth',
          label: provider.auth.oauth.loginLabel ?? provider.auth.oauth.name,
          interactive: true,
          persistentAllowed: this.persistenceAvailable(),
        }]
        : [];
      return {
        id: provider.id,
        name: provider.name,
        authentication,
        authState: check ? 'connected' : (stored ? 'configured' : 'disconnected'),
        authType: check?.type ?? stored?.type ?? null,
        authSource: check?.source ?? (stored?.type === 'oauth' ? 'Stored OAuth' : null),
      };
      }));
    const models = this.modelRuntime.getModels()
      .filter((model) => model.provider === FRAIA_AI_PROVIDER_ID && model.id === FRAIA_AI_MODEL_ID)
      .map((model) => ({
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
      const result = await this.modelRuntime.refresh({ allowNetwork: false });
      this.catalogRefreshError = refreshErrorMessage(result);
      if (this.catalogRefreshError) {
        this.emitStatus({ kind: 'catalogue', state: 'stale', reason, message: this.catalogRefreshError });
      } else {
        this.catalogRefreshedAt = new Date().toISOString();
        this.emitStatus({ kind: 'catalogue', state: 'refreshed', reason, at: this.catalogRefreshedAt });
      }
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

  async startOAuth(providerId) {
    if (providerId !== FRAIA_AI_PROVIDER_ID) {
      throw new Error(`Fraia supports only the ${FRAIA_AI_PROVIDER_ID} OAuth connection.`);
    }
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
    if (providerId !== FRAIA_AI_PROVIDER_ID) {
      throw new Error(`Fraia supports only the ${FRAIA_AI_PROVIDER_ID} OAuth connection.`);
    }
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
    if (
      providerId !== FRAIA_AI_PROVIDER_ID
      || modelId !== FRAIA_AI_MODEL_ID
      || reasoningEffort !== 'low'
    ) {
      throw new Error(`Fraia supports only ${FRAIA_AI_PROVIDER_ID}/${FRAIA_AI_MODEL_ID} with low reasoning.`);
    }
    const model = this.modelRuntime.getModel(providerId, modelId);
    if (!model) throw new Error(`Pi does not know model ${providerId}/${modelId}.`);
    const available = await this.modelRuntime.getAvailable(providerId);
    if (!available.some((candidate) => candidate.id === modelId)) {
      throw new Error(`Selected model ${providerId}/${modelId} is unavailable. Choose an available authenticated model.`);
    }

    let structuredResult = null;
    const schema = typeBoxSchema(this.Type, responseSchema);
    const tool = {
      name: 'submit_fraia_response',
      label: 'Submit Fraia response',
      description: 'Submit the final structured response to Fraia. This must be the final action.',
      parameters: schema,
      async execute(_toolCallId, params) {
        structuredResult = params;
        return {
          content: [{ type: 'text', text: 'Fraia received the structured response.' }],
          details: { accepted: true },
          terminate: true,
        };
      },
    };
    const agent = new this.pi.Agent({
      initialState: {
        systemPrompt: 'You are Fraia\'s constrained AI reasoning adapter. Use only the supplied prompt. Do not infer access to files, tools, repositories, or project context. Always finish by calling submit_fraia_response exactly once with arguments matching its schema. Do not provide the final result as unstructured assistant text.',
        model,
        thinkingLevel: reasoningEffort,
        tools: [tool],
      },
      streamFn: (selectedModel, context, options) => this.modelRuntime.streamSimple(selectedModel, context, options),
      toolExecution: 'sequential',
    });
    const activeTurn = { agent, cancellationMessage: null };
    const timeout = setTimeout(() => {
      activeTurn.cancellationMessage = 'The AI turn timed out and was cancelled.';
      agent.abort();
    }, this.turnTimeoutMs);
    this.activeTurns.set(requestId, activeTurn);
    this.emitStatus({ kind: 'turn', requestId, state: 'generating', providerId, modelId });
    const promptAgent = async (message) => {
      if (activeTurn.cancellationMessage) throw new Error(activeTurn.cancellationMessage);
      try {
        await agent.prompt(message);
      } catch (error) {
        if (activeTurn.cancellationMessage) throw new Error(activeTurn.cancellationMessage);
        throw error;
      }
      if (activeTurn.cancellationMessage) throw new Error(activeTurn.cancellationMessage);
      if (agent.state.errorMessage) throw new Error(agent.state.errorMessage);
    };
    try {
      await promptAgent(prompt);
      if (!structuredResult) {
        this.emitStatus({ kind: 'turn', requestId, state: 'correcting' });
        await promptAgent('Your previous response did not call submit_fraia_response with valid arguments. Call it now with a complete response matching the tool schema.');
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
      agent.reset();
    }
  }

  async cancelTurn(requestId) {
    const activeTurn = this.activeTurns.get(requestId);
    if (!activeTurn) return false;
    activeTurn.cancellationMessage = 'The AI turn was cancelled.';
    activeTurn.agent.abort();
    await activeTurn.agent.waitForIdle();
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
    if (this.server) {
      this.server.closeAllConnections?.();
      await new Promise((resolve) => this.server.close(resolve));
    }
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
    const credentialCipher = this.safeStorage?.isEncryptionAvailable?.()
      ? this.safeStorage
      : fakeAiTestSafeStorage();
    this.credentials = new SecureCredentialStore({ safeStorage: credentialCipher, filePath: credentialFile });
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

  async cancelTurn(requestId) {
    const turn = this.activeTurns.get(requestId);
    if (!turn) return false;
    await turn.abort();
    this.emitStatus({ kind: 'turn', requestId, state: 'cancelled' });
    return true;
  }
}

module.exports = {
  FakeFraiaAiRuntime,
  FraiaAiRuntime,
  NonPersistentCredentialStore,
  SecureCredentialStore,
  fakeAiTestSafeStorage,
  publicFraiaCatalogue,
  reasoningLevels,
  typeBoxSchema,
  fakeValueForSchema,
};
