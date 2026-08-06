const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const {
  FakeFraiaAiRuntime,
  FraiaAiRuntime,
  NonPersistentCredentialStore,
  SecureCredentialStore,
  fakeAiTestSafeStorage,
  publicFraiaCatalogue,
  reasoningLevels,
  typeBoxSchema,
} = require('../ai-runtime.cjs');

function temporaryFile() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-ai-runtime-test-'));
  return { directory, filePath: path.join(directory, 'credentials.bin') };
}

function fakeSafeStorage(available = true) {
  return {
    isEncryptionAvailable: () => available,
    encryptString: (value) => Buffer.from(`encrypted:${Buffer.from(value).toString('base64')}`),
    decryptString: (value) => Buffer.from(value.toString().slice('encrypted:'.length), 'base64').toString(),
  };
}

test('secure credential store provides encrypted CRUD without writing plaintext', async (t) => {
  const { directory, filePath } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const store = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath });
  const secret = 'fraia-test-secret-never-on-disk';

  await store.modify('provider-a', async () => ({ type: 'api_key', key: secret }));
  assert.deepEqual(await store.read('provider-a'), { type: 'api_key', key: secret });
  assert.deepEqual(await store.list(), [{ providerId: 'provider-a', type: 'api_key' }]);
  assert.equal(fs.readFileSync(filePath).includes(Buffer.from(secret)), false);

  await store.delete('provider-a');
  assert.equal(await store.read('provider-a'), undefined);
});

test('secure credential store serializes concurrent provider writes', async (t) => {
  const { directory, filePath } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const store = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath });

  await Promise.all([
    store.modify('provider-a', async () => ({ type: 'api_key', key: 'a' })),
    store.modify('provider-b', async () => ({ type: 'oauth', access: 'b', refresh: 'r', expires: 1 })),
  ]);

  assert.equal((await store.read('provider-a')).key, 'a');
  assert.equal((await store.read('provider-b')).access, 'b');
});

test('secure credential store rejects unavailable encryption and corrupted data', async (t) => {
  const first = temporaryFile();
  const second = temporaryFile();
  t.after(() => {
    fs.rmSync(first.directory, { recursive: true, force: true });
    fs.rmSync(second.directory, { recursive: true, force: true });
  });
  const unavailable = new SecureCredentialStore({ safeStorage: fakeSafeStorage(false), filePath: first.filePath });
  await assert.rejects(() => unavailable.modify('provider', async () => ({ type: 'api_key', key: 'secret' })), /will not store credentials as plaintext/);
  assert.equal(fs.existsSync(first.filePath), false);

  fs.writeFileSync(second.filePath, 'not-encrypted-json');
  const corrupted = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath: second.filePath });
  await assert.rejects(() => corrupted.read('provider'), /could not decrypt its AI credential store/);
});

test('non-persistent store exposes no credentials and refuses writes', async () => {
  const store = new NonPersistentCredentialStore();
  assert.equal(await store.read('provider'), undefined);
  assert.deepEqual(await store.list(), []);
  await assert.rejects(() => store.modify('provider', async () => ({ type: 'api_key', key: 'secret' })), /persistent authentication is disabled/);
});

test('runtime closes active HTTP connections before waiting for server shutdown', async () => {
  const runtime = new FraiaAiRuntime({ safeStorage: fakeSafeStorage(), userDataDir: '/tmp/fraia-unused' });
  let activeConnectionsClosed = false;
  let serverClosed = false;
  runtime.server = {
    closeAllConnections() {
      activeConnectionsClosed = true;
    },
    close(callback) {
      assert.equal(activeConnectionsClosed, true);
      serverClosed = true;
      callback();
    },
  };

  await runtime.stop();

  assert.equal(serverClosed, true);
  assert.equal(runtime.server, null);
});

test('fake AI test cipher persists only fake tokens without relying on OS encryption', async () => {
  const first = fakeAiTestSafeStorage();
  const second = fakeAiTestSafeStorage();
  const plaintext = 'fake-chatgpt-access-token';
  const encrypted = first.encryptString(plaintext);

  assert.equal(encrypted.includes(Buffer.from(plaintext)), false);
  assert.equal(second.decryptString(encrypted), plaintext);
  assert.throws(
    () => second.decryptString(Buffer.from(encrypted).fill(0, encrypted.length - 1)),
    /invalid fake AI credential ciphertext|Unsupported state or unable to authenticate data/,
  );
});

test('catalogue reasoning levels reflect model capability', () => {
  assert.deepEqual(reasoningLevels({ reasoning: false }), [
    { effort: 'off', description: 'Reasoning is not exposed for this model.' },
  ]);
  assert.deepEqual(
    reasoningLevels({ reasoning: true, thinkingLevelMap: { minimal: 'low', xhigh: 'xhigh' } })
      .map((item) => item.effort),
    ['off', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'],
  );
});

test('public catalogue exposes only the reviewed ChatGPT Luna contract', () => {
  const result = publicFraiaCatalogue({
    providers: [
      { id: 'anthropic', name: 'Anthropic' },
      { id: 'openai-codex', name: 'OpenAI Codex' },
    ],
    models: [
      { providerId: 'anthropic', modelId: 'claude-sonnet' },
      { providerId: 'openai-codex', modelId: 'gpt-5.5' },
      { providerId: 'openai-codex', modelId: 'gpt-5.6-luna' },
    ],
    catalogue: { source: 'test' },
    secureCredentialStorageAvailable: true,
  });

  assert.deepEqual(result.providers.map((provider) => provider.id), ['openai-codex']);
  assert.deepEqual(result.models.map((model) => model.modelId), ['gpt-5.6-luna']);
  assert.deepEqual(result.catalogue, { source: 'test' });
  assert.equal(result.secureCredentialStorageAvailable, true);
});

test('renderer bridge exposes no API-key or model-setting mutation', () => {
  const preload = fs.readFileSync(path.join(__dirname, '..', 'preload.js'), 'utf8');
  const main = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');

  assert.doesNotMatch(preload, /aiSubmitApiKey|agentUpdateSettings/);
  assert.doesNotMatch(main, /ipcMain\.handle\(['"]fraia:(?:aiSubmitApiKey|agentUpdateSettings)/);
});

test('structured schema conversion supports nullable enums', async () => {
  const { Type } = await import('typebox');
  const schema = typeBoxSchema(Type, {
    type: ['string', 'null'],
    enum: ['active', 'superseded', null],
  });

  assert.deepEqual(schema.anyOf.map((item) => item.type), ['string', 'string', 'null']);
  assert.deepEqual(schema.anyOf.map((item) => item.const), ['active', 'superseded', undefined]);
});

test('runtime initializes only the reviewed Pi provider with encrypted credentials and offline catalogue refresh', async (t) => {
  const { directory } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  let createOptions;
  let registeredProvider;
  let refreshOptions;
  const modelRuntime = {
    setProvider: (provider) => { registeredProvider = provider; },
    refresh: async (options) => {
      refreshOptions = options;
      return { aborted: false, errors: new Map() };
    },
  };
  const runtime = new FraiaAiRuntime({
    safeStorage: fakeSafeStorage(),
    userDataDir: directory,
    importPi: async () => ({
      createModels: (options) => {
        createOptions = options;
        return modelRuntime;
      },
      openaiCodexProvider: () => ({ id: 'openai-codex', name: 'OpenAI Codex' }),
    }),
    importTypeBox: async () => ({ Type: {} }),
  });
  await runtime.initialize();
  t.after(() => runtime.stop());

  assert.equal(createOptions.credentials instanceof SecureCredentialStore, true);
  assert.equal(registeredProvider.id, 'openai-codex');
  assert.deepEqual(refreshOptions, { allowNetwork: false });
});

test('pinned production Pi packages expose the reviewed ChatGPT Luna catalogue', async (t) => {
  const { directory } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const runtime = await new FraiaAiRuntime({
    safeStorage: fakeSafeStorage(),
    userDataDir: directory,
  }).initialize();
  t.after(() => runtime.stop());

  const result = await runtime.catalog();
  assert.deepEqual(result.providers.map((provider) => provider.id), ['openai-codex']);
  assert.deepEqual(
    result.models.map((model) => [model.providerId, model.modelId]),
    [['openai-codex', 'gpt-5.6-luna']],
  );
  assert.equal(result.providers[0].authentication[0].type, 'oauth');
  assert.equal(result.providers[0].authState, 'disconnected');
  assert.equal(result.models[0].available, false);
  assert.equal(result.catalogue.stale, false);
});

test('catalogue describes the reviewed ChatGPT OAuth provider and Luna model', async () => {
  const runtime = new FraiaAiRuntime({ safeStorage: fakeSafeStorage(), userDataDir: '/tmp/fraia-unused' });
  runtime.credentials = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath: '/tmp/fraia-unused-credentials' });
  runtime.catalogRefreshedAt = '2026-07-22T00:00:00Z';
  runtime.modelRuntime = {
    getAvailable: async () => [{ provider: 'openai-codex', id: 'gpt-5.6-luna' }],
    getProviders: () => [
      { id: 'openai-codex', name: 'OpenAI Codex', auth: { oauth: { name: 'OpenAI (ChatGPT Plus/Pro)', loginLabel: 'Sign in with ChatGPT' } } },
    ],
    checkAuth: async () => ({ type: 'oauth', source: 'OAuth' }),
    getModels: () => [{
      provider: 'openai-codex',
      id: 'gpt-5.6-luna',
      name: 'GPT-5.6 Luna',
      reasoning: true,
      contextWindow: 272000,
      maxTokens: 128000,
    }],
  };

  const result = await runtime.catalog();
  assert.deepEqual(result.providers.map((provider) => provider.id), ['openai-codex']);
  assert.equal(result.providers[0].authentication[0].type, 'oauth');
  assert.equal(result.providers[0].authState, 'connected');
  assert.equal(result.models[0].modelId, 'gpt-5.6-luna');
  assert.equal(result.models[0].available, true);
  assert.equal(result.models[0].defaultReasoningLevel, 'low');
});

test('OAuth flow automatically chooses browser login, opens its URL, and keeps the manual fallback out of Fraia', async (t) => {
  const { directory, filePath } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const events = [];
  const opened = [];
  const runtime = new FraiaAiRuntime({
    safeStorage: fakeSafeStorage(),
    shell: { openExternal: async (url) => opened.push(url) },
    userDataDir: directory,
    emitStatus: (event) => events.push(event),
  });
  runtime.credentials = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath });
  runtime.modelRuntime = {
    login: async (_providerId, type, interaction) => {
      assert.equal(type, 'oauth');
      const loginMethod = await interaction.prompt({
        type: 'select',
        message: 'Select OpenAI Codex login method:',
        options: [
          { id: 'browser', label: 'Browser login (default)' },
          { id: 'device_code', label: 'Device code login (headless)' },
        ],
      });
      assert.equal(loginMethod, 'browser');
      interaction.notify({ type: 'auth_url', url: 'https://provider.example/sign-in' });
      const manualAbort = new AbortController();
      const ignoredManualPrompt = interaction.prompt({
        type: 'manual_code',
        message: 'Paste the redirect URL',
        signal: manualAbort.signal,
      }).catch(() => {});
      manualAbort.abort();
      await ignoredManualPrompt;
    },
    refresh: async () => ({ aborted: false, errors: new Map() }),
    getAvailable: async () => [],
    getProviders: () => [],
    getModels: () => [],
  };

  await runtime.startOAuth('openai-codex');
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(opened, ['https://provider.example/sign-in']);
  assert.equal(events.some((event) => event.type === 'prompt'), false);
  assert.equal(events.some((event) => event.type === 'complete'), true);
});

test('OAuth flow rejects an unexpected device-code branch without opening it', async (t) => {
  const { directory, filePath } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const events = [];
  const opened = [];
  const runtime = new FraiaAiRuntime({
    safeStorage: fakeSafeStorage(),
    shell: { openExternal: async (url) => opened.push(url) },
    userDataDir: directory,
    emitStatus: (event) => events.push(event),
  });
  runtime.credentials = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath });
  runtime.modelRuntime = {
    login: async (_providerId, _type, interaction) => {
      interaction.notify({ type: 'device_code', verificationUri: 'https://provider.example/device', userCode: 'ABCD' });
    },
    refresh: async () => ({ aborted: false, errors: new Map() }),
    getAvailable: async () => [],
    getProviders: () => [],
    getModels: () => [],
  };

  await runtime.startOAuth('openai-codex');
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(opened, []);
  assert.match(events.find((event) => event.type === 'error')?.message ?? '', /browser sign-in only/);
});

test('catalogue refresh retains stale state and diagnostics after an offline failure', async () => {
  const runtime = new FraiaAiRuntime({ safeStorage: fakeSafeStorage(), userDataDir: '/tmp/fraia-unused' });
  runtime.credentials = new SecureCredentialStore({ safeStorage: fakeSafeStorage(), filePath: '/tmp/fraia-unused-credentials' });
  runtime.catalogRefreshedAt = '2026-07-21T00:00:00Z';
  runtime.modelRuntime = {
    refresh: async () => ({ aborted: false, errors: new Map([['openai-codex', new Error('offline')]]) }),
    getAvailable: async () => [],
    getProviders: () => [],
    getModels: () => [{ provider: 'openai-codex', id: 'gpt-5.6-luna', name: 'GPT-5.6 Luna', reasoning: true }],
  };
  const result = await runtime.refreshCatalog('manual');
  assert.equal(result.catalogue.stale, true);
  assert.match(result.catalogue.refreshError, /offline/);
  assert.equal(result.catalogue.refreshedAt, '2026-07-21T00:00:00Z');
  assert.equal(result.models[0].available, false);
});

async function structuredRuntime({ behavior = 'valid', turnTimeoutMs = 500 } = {}) {
  const Type = (await import('typebox')).Type;
  let promptCalls = 0;
  let rejectPending;
  const pi = {
    Agent: class {
      constructor({ initialState }) {
        this.state = { errorMessage: undefined };
        this.tools = initialState.tools;
      }

      async prompt() {
          promptCalls += 1;
          if (behavior === 'provider-error') {
            this.state.errorMessage = 'provider unavailable';
            return;
          }
          if (behavior === 'timeout') {
            return new Promise((_resolve, reject) => { rejectPending = reject; });
          }
          if (behavior === 'valid' || (behavior === 'corrective' && promptCalls === 2)) {
            await this.tools[0].execute('tool-call', { message: 'Structured result' });
          }
      }

      abort() {
        rejectPending?.(new Error('aborted'));
      }

      waitForIdle() {
        return Promise.resolve();
      }

      reset() {}
    },
  };
  const runtime = new FraiaAiRuntime({ safeStorage: fakeSafeStorage(), userDataDir: '/tmp/fraia-structured-runtime', turnTimeoutMs });
  runtime.pi = pi;
  runtime.Type = Type;
  runtime.catalogRefreshedAt = '2026-07-22T00:00:00Z';
  runtime.modelRuntime = {
    getModel: (providerId, modelId) => ({ provider: providerId, id: modelId }),
    getAvailable: async () => [{ provider: 'openai-codex', id: 'gpt-5.6-luna' }],
    streamSimple: () => {
      throw new Error('The fake Agent must not call the production stream.');
    },
  };
  return { runtime, promptCalls: () => promptCalls };
}

test('structured turns use only the reviewed ChatGPT Luna contract', async () => {
  const { runtime } = await structuredRuntime();
  const result = await runtime.runTurn({
    requestId: 'turn-luna',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Return structured data',
    responseSchema: {
      type: 'object',
      properties: { message: { type: 'string' } },
      required: ['message'],
      additionalProperties: false,
    },
  });
  assert.deepEqual(result.output, { message: 'Structured result' });
  assert.equal(result.providerId, 'openai-codex');
  await assert.rejects(() => runtime.runTurn({
    requestId: 'turn-unreviewed',
    providerId: 'anthropic',
    modelId: 'claude-opus',
    reasoningEffort: 'low',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /supports only openai-codex\/gpt-5.6-luna with low reasoning/);
});

test('structured turns make one corrective attempt and reject missing tool results', async () => {
  const corrective = await structuredRuntime({ behavior: 'corrective' });
  const result = await corrective.runtime.runTurn({
    requestId: 'turn-corrective',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: { message: { type: 'string' } }, required: ['message'] },
  });
  assert.equal(result.output.message, 'Structured result');
  assert.equal(corrective.promptCalls(), 2);

  const invalid = await structuredRuntime({ behavior: 'missing-tool' });
  await assert.rejects(() => invalid.runtime.runTurn({
    requestId: 'turn-invalid',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /did not submit a valid structured Fraia response/);
  assert.equal(invalid.promptCalls(), 2);
});

test('structured turns surface provider failures and enforce the configured timeout', async () => {
  const providerFailure = await structuredRuntime({ behavior: 'provider-error' });
  await assert.rejects(() => providerFailure.runtime.runTurn({
    requestId: 'turn-provider-error',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /provider unavailable/);

  const timeout = await structuredRuntime({ behavior: 'timeout', turnTimeoutMs: 5 });
  await assert.rejects(() => timeout.runtime.runTurn({
    requestId: 'turn-timeout',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /cancelled/);
});

test('fake Pi runtime covers encrypted reconnect without OS encryption, structured turns, cancellation, and restart', async (t) => {
  const { directory } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const events = [];
  const options = {
    safeStorage: fakeSafeStorage(false),
    userDataDir: directory,
    emitStatus: (event) => events.push(event),
  };
  const runtime = await new FakeFraiaAiRuntime(options).initialize();
  assert.equal((await runtime.catalog()).providers[0].authState, 'disconnected');
  await runtime.startOAuth('openai-codex');
  assert.equal((await runtime.catalog()).providers[0].authState, 'connected');
  assert.equal(events.some((event) => event.kind === 'authentication' && event.type === 'complete'), true);
  const result = await runtime.runTurn({
    requestId: 'turn-complete',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Return a result',
    responseSchema: {
      type: 'object',
      properties: { message: { type: 'string' } },
      required: ['message'],
      additionalProperties: false,
    },
  });
  assert.deepEqual(result.output, { message: 'Fake Pi response' });

  process.env.FRAIA_FAKE_AI_TURN_DELAY_MS = '5000';
  const pending = runtime.runTurn({
    requestId: 'turn-cancel',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'low',
    prompt: 'Wait',
    responseSchema: { type: 'object', properties: {}, required: [] },
  });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(await runtime.cancelTurn('turn-cancel'), true);
  await assert.rejects(() => pending, /cancelled/);
  delete process.env.FRAIA_FAKE_AI_TURN_DELAY_MS;

  const restarted = await new FakeFraiaAiRuntime(options).initialize();
  assert.equal((await restarted.catalog()).providers[0].authState, 'connected');
  const encryptedCredential = fs.readFileSync(path.join(directory, 'ai', 'credentials.bin'));
  assert.equal(encryptedCredential.includes(Buffer.from('fake-chatgpt-access-token')), false);
  assert.equal(encryptedCredential.includes(Buffer.from('fake-chatgpt-refresh-token')), false);
});
