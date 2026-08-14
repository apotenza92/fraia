const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const connectedQualification = require('./fixtures/connected-qualification-request.json');

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

test('runtime stops accepting AI connections before closing active HTTP connections', async () => {
  const runtime = new FraiaAiRuntime({ safeStorage: fakeSafeStorage(), userDataDir: '/tmp/fraia-unused' });
  let serverCloseStarted = false;
  let activeConnectionsClosed = false;
  let serverClosed = false;
  runtime.server = {
    closeAllConnections() {
      assert.equal(serverCloseStarted, true);
      activeConnectionsClosed = true;
    },
    close(callback) {
      serverCloseStarted = true;
      serverClosed = true;
      queueMicrotask(() => callback());
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

test('renderer bridge exposes no legacy direct base-model mutation', () => {
  const preload = fs.readFileSync(path.join(__dirname, '..', 'preload.js'), 'utf8');
  const main = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');

  assert.doesNotMatch(preload, /editBaseModel|fraia:editBaseModel/);
  assert.doesNotMatch(main, /fraia:editBaseModel|projects\/base-model\/edit/);
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
  assert.equal(result.models[0].defaultReasoningLevel, 'high');
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

async function structuredRuntime({ behavior = 'valid', turnTimeoutMs = 500, turnHeartbeatMs = 5_000 } = {}) {
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
  const events = [];
  const runtime = new FraiaAiRuntime({
    safeStorage: fakeSafeStorage(),
    userDataDir: '/tmp/fraia-structured-runtime',
    turnTimeoutMs,
    turnHeartbeatMs,
    emitStatus: (event) => events.push(event),
  });
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
  return { runtime, events, promptCalls: () => promptCalls };
}

test('structured turns use only the reviewed ChatGPT Luna contract', async () => {
  const { runtime } = await structuredRuntime();
  const result = await runtime.runTurn({
    requestId: 'turn-luna',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
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
  assert.deepEqual(
    runtime.activeTurns.size,
    0,
  );
  await assert.rejects(() => runtime.runTurn({
    requestId: 'turn-unreviewed',
    providerId: 'anthropic',
    modelId: 'claude-opus',
    reasoningEffort: 'high',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /supports only openai-codex\/gpt-5.6-luna with high reasoning/);
});

test('structured turns make one corrective attempt and reject missing tool results', async () => {
  const corrective = await structuredRuntime({ behavior: 'corrective' });
  const result = await corrective.runtime.runTurn({
    requestId: 'turn-corrective',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
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
    reasoningEffort: 'high',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /did not submit a valid structured Fraia response/);
  assert.equal(invalid.promptCalls(), 2);
});

test('structured turns emit one truthful terminal state for failure and timeout', async () => {
  const providerFailure = await structuredRuntime({ behavior: 'provider-error' });
  await assert.rejects(() => providerFailure.runtime.runTurn({
    requestId: 'turn-provider-error',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /provider unavailable/);
  assert.deepEqual(
    providerFailure.events.filter((event) => ['completed', 'failed', 'cancelled', 'timed_out'].includes(event.state)).map((event) => event.state),
    ['failed'],
  );

  const timeout = await structuredRuntime({ behavior: 'timeout', turnTimeoutMs: 10, turnHeartbeatMs: 2 });
  await assert.rejects(() => timeout.runtime.runTurn({
    requestId: 'turn-timeout',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /timed out/);
  assert.equal(timeout.events.some((event) => event.state === 'working' && event.liveness === true), true);
  assert.deepEqual(
    timeout.events.filter((event) => ['completed', 'failed', 'cancelled', 'timed_out'].includes(event.state)).map((event) => event.state),
    ['timed_out'],
  );
});

test('structured turns enforce the caller shared absolute deadline', async () => {
  const timeout = await structuredRuntime({ behavior: 'timeout', turnTimeoutMs: 500 });
  const startedAt = Date.now();
  await assert.rejects(() => timeout.runtime.runTurn({
    requestId: 'turn-shared-deadline-correction',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    deadlineAtUnixMs: startedAt + 12,
    prompt: 'Return corrected structured data within the original turn deadline',
    responseSchema: { type: 'object', properties: {}, required: [] },
  }), /timed out/);
  assert.ok(Date.now() - startedAt < 250, 'the request must not receive a fresh 500 ms budget');
  assert.deepEqual(
    timeout.events.filter((event) => ['completed', 'failed', 'cancelled', 'timed_out'].includes(event.state)).map((event) => event.state),
    ['timed_out'],
  );
});

test('production turn cancellation rejects late completion and active request-id reuse', async () => {
  const cancelled = await structuredRuntime({ behavior: 'timeout', turnTimeoutMs: 500 });
  const request = {
    requestId: 'turn-cancelled',
    scopeId: 'design-cancelled',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: 'Return structured data',
    responseSchema: { type: 'object', properties: {}, required: [] },
  };
  const pending = cancelled.runtime.runTurn(request);
  await new Promise((resolve) => setImmediate(resolve));
  await assert.rejects(() => cancelled.runtime.runTurn(request), /already active/);
  assert.equal(await cancelled.runtime.cancelTurnsForScope('design-cancelled'), 1);
  await assert.rejects(() => pending, /cancelled/);
  assert.deepEqual(
    cancelled.events.filter((event) => ['completed', 'failed', 'cancelled', 'timed_out'].includes(event.state)).map((event) => event.state),
    ['cancelled'],
  );
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
    reasoningEffort: 'high',
    prompt: 'Return a result',
    responseSchema: {
      type: 'object',
      properties: { message: { type: 'string' } },
      required: ['message'],
      additionalProperties: false,
    },
  });
  assert.deepEqual(result.output, { message: 'Fake Pi response' });

  const typed = await runtime.runTurn({
    requestId: 'turn-typed-proposal',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: 'FRAIA_FAKE_TYPED_PROPOSAL_REQUEST {"acceptedHeadRevisionId":"exact-head","acceptedSnapshotId":"exact-snapshot","selectedDesignReferenceIds":["reference-1"],"drawingInterpretationRevisionIds":["interpretation-1"],"inferredDrawingAssumptionIds":["interpretation-1:inference:grid-a"],"inferredDrawingAssumptions":["Inferred drawing candidate interpretation-1:inference:grid-a has confidence 0.900, requires confirmation, and is not a confirmed fact."]}',
    responseSchema: { type: 'object', properties: {}, required: [] },
  });
  assert.equal(typed.output.proposal.parentRevisionId, 'exact-head');
  assert.equal(typed.output.proposal.expectedSnapshotId, 'exact-snapshot');
  assert.deepEqual(typed.output.proposal.shelfItemIds, ['reference-1']);
  assert.deepEqual(typed.output.proposal.drawingInterpretationRevisionIds, ['interpretation-1']);
  assert.deepEqual(typed.output.proposal.drawingInterpretationInferenceIds, [
    'interpretation-1:inference:grid-a',
  ]);
  assert.equal(
    typed.output.proposal.assumptions.includes(
      'Inferred drawing candidate interpretation-1:inference:grid-a has confidence 0.900, requires confirmation, and is not a confirmed fact.',
    ),
    true,
  );
  assert.match(typed.output.proposal.evidenceLimits[0], /requires confirmation/);
  assert.equal(typed.output.proposal.operations.some((operation) => operation.kind === 'add_member'), true);
  assert.deepEqual(
    typed.output.proposal.operations.filter((operation) => operation.kind === 'add_support').map((operation) => operation.targetNode),
    ['test-left', 'test-right'],
  );
  assert.deepEqual(
    events.filter((event) => event.requestId === 'turn-typed-proposal').map((event) => event.state),
    ['sending', 'working', 'checking', 'completed'],
  );

  process.env.FRAIA_FAKE_AI_MALFORMED_FIRST_RESPONSE = '1';
  try {
    // This keeps the same sorted key shape emitted by serde_json in appd. The
    // contract key follows nested objects, so the parser must find the outer
    // context object rather than the nearest opening brace.
    const exactContext = JSON.stringify({
      acceptedHeadRevisionId: 'exact-head',
      acceptedSemanticModel: { nodes: [], members: [] },
      acceptedSnapshotId: 'exact-snapshot',
      confirmedDrawingInterpretations: [{ revisionId: 'interpretation-1', confirmedConstraints: [] }],
      confirmedFacts: { buildingType: 'house' },
      contract: 'fraia.conversation-agent.v1',
      drawingInterpretationRevisionIds: ['interpretation-1'],
      inferredDrawingAssumptionIds: ['interpretation-1:inference:grid-a'],
      inferredDrawingAssumptions: ['Candidate grid-a requires confirmation.'],
      requestMarker: 'FRAIA_FAKE_TYPED_PROPOSAL_REQUEST',
      selectedDesignReferenceIds: ['dxf-selection-1', 'ifc-selection-1', 'mesh-view-1'],
      selectedConfirmedDesignReferences: [{ id: 'dxf-selection-1' }],
    }, null, 2);
    const malformed = await runtime.runTurn({
      requestId: 'turn-malformed-first',
      providerId: 'openai-codex',
      modelId: 'gpt-5.6-luna',
      reasoningEffort: 'high',
      prompt: `Use this exact context.\n${exactContext}`,
      responseSchema: { type: 'object', properties: {}, required: [] },
    });
    assert.equal(malformed.output.proposal.operations[3].nodeId, 'test-left');
    assert.equal(malformed.output.proposal.operations[3].targetNode, undefined);
    const corrected = await runtime.runTurn({
      requestId: 'turn-malformed-first:schema-correction',
      providerId: 'openai-codex',
      modelId: 'gpt-5.6-luna',
      reasoningEffort: 'high',
      prompt: `Your previous response failed validation.\nExact schema:\n{"type":"object"}\nRejected response:\n{"proposal":{}}\nOriginal request:\nUse this exact context.\n${exactContext}`,
      responseSchema: { type: 'object', properties: {}, required: [] },
    });
    assert.equal(corrected.output.proposal.operations[3].targetNode, 'test-left');
    assert.equal(corrected.output.proposal.operations[3].nodeId, undefined);
    assert.equal(corrected.output.proposal.parentRevisionId, 'exact-head');
    assert.equal(corrected.output.proposal.expectedSnapshotId, 'exact-snapshot');
    assert.deepEqual(corrected.output.proposal.shelfItemIds, ['dxf-selection-1', 'ifc-selection-1', 'mesh-view-1']);
    assert.deepEqual(corrected.output.proposal.drawingInterpretationRevisionIds, ['interpretation-1']);
    assert.deepEqual(corrected.output.proposal.drawingInterpretationInferenceIds, ['interpretation-1:inference:grid-a']);
  } finally {
    delete process.env.FRAIA_FAKE_AI_MALFORMED_FIRST_RESPONSE;
  }

  const conversational = await runtime.runTurn({
    requestId: 'turn-conversational-no-proposal',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: 'Discuss the current design without the explicit typed proposal test marker.',
    responseSchema: {
      type: 'object',
      properties: {
        responseId: { type: 'string' },
        text: { type: 'string' },
        questions: { type: 'array', items: { type: 'string' } },
        proposal: { type: ['object', 'null'] },
      },
      required: ['responseId', 'text', 'questions'],
    },
  });
  assert.match(conversational.output.text, /confirmed dimensions/);
  assert.equal(conversational.output.proposal, undefined);

  process.env.FRAIA_FAKE_AI_TURN_DELAY_MS = '5000';
  const pending = runtime.runTurn({
    requestId: 'turn-cancel',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: 'Wait',
    responseSchema: { type: 'object', properties: {}, required: [] },
  });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(await runtime.cancelTurn('turn-cancel'), true);
  await assert.rejects(() => pending, /cancelled/);
  assert.deepEqual(
    events.filter((event) => event.requestId === 'turn-cancel' && ['completed', 'failed', 'cancelled', 'timed_out'].includes(event.state)).map((event) => event.state),
    ['cancelled'],
  );

  const scopedFirst = runtime.runTurn({
    requestId: 'turn-scope-first', scopeId: 'design-a', providerId: 'openai-codex', modelId: 'gpt-5.6-luna', reasoningEffort: 'high', prompt: 'Wait', responseSchema: { type: 'object', properties: {}, required: [] },
  });
  const scopedSecond = runtime.runTurn({
    requestId: 'turn-scope-second', scopeId: 'design-b', providerId: 'openai-codex', modelId: 'gpt-5.6-luna', reasoningEffort: 'high', prompt: 'Wait', responseSchema: { type: 'object', properties: {}, required: [] },
  });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(await runtime.cancelTurnsForScope('design-a'), 1);
  await assert.rejects(() => scopedFirst, /cancelled/);
  assert.equal(runtime.activeTurns.has('turn-scope-second'), true);
  assert.equal(await runtime.cancelTurn('turn-scope-second'), true);
  await assert.rejects(() => scopedSecond, /cancelled/);
  delete process.env.FRAIA_FAKE_AI_TURN_DELAY_MS;

  const restarted = await new FakeFraiaAiRuntime(options).initialize();
  assert.equal((await restarted.catalog()).providers[0].authState, 'connected');
  const encryptedCredential = fs.readFileSync(path.join(directory, 'ai', 'credentials.bin'));
  assert.equal(encryptedCredential.includes(Buffer.from('fake-chatgpt-access-token')), false);
  assert.equal(encryptedCredential.includes(Buffer.from('fake-chatgpt-refresh-token')), false);
});

test('connected qualification request has deterministic typed fake parity without a visible test marker', async (t) => {
  assert.doesNotMatch(connectedQualification.request, /FRAIA_FAKE/);
  for (const explicitFact of ['6 metre', 'beam', '250UB', 'steel', 'pinned support', 'roller support', 'Do not add loads']) {
    assert.match(connectedQualification.request, new RegExp(explicitFact, 'i'));
  }
  const { directory } = temporaryFile();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const runtime = await new FakeFraiaAiRuntime({ safeStorage: fakeSafeStorage(false), userDataDir: directory }).initialize();
  await runtime.startOAuth('openai-codex');
  const context = {
    contract: 'fraia.conversation-agent.v1',
    requestMarker: 'FRAIA_FAKE_TYPED_PROPOSAL_REQUEST',
    acceptedHeadRevisionId: 'qualification-head',
    acceptedSnapshotId: 'qualification-snapshot',
    acceptedSemanticModel: { nodes: [], members: [] },
    selectedDesignReferenceIds: [],
    drawingInterpretationRevisionIds: [],
    inferredDrawingAssumptionIds: [],
    inferredDrawingAssumptions: [],
    userText: connectedQualification.request,
  };
  const response = await runtime.runTurn({
    requestId: 'connected-qualification-parity',
    scopeId: 'qualification-design',
    providerId: 'openai-codex',
    modelId: 'gpt-5.6-luna',
    reasoningEffort: 'high',
    prompt: `Use this reviewed context.\n${JSON.stringify(context, null, 2)}`,
    responseSchema: { type: 'object', properties: { responseId: {}, text: {}, proposal: {} }, required: [] },
  });
  assert.doesNotMatch(response.output.text, /FRAIA_FAKE/);
  assert.deepEqual(response.output.proposal.operations, [
    { kind: 'add_node', id: 'test-left', x: 0, y: 0, z: 0 },
    { kind: 'add_node', id: 'test-right', x: 6, y: 0, z: 0 },
    { kind: 'add_member', id: 'test-beam', startNode: 'test-left', endNode: 'test-right', role: 'beam', sectionId: '250UB', materialId: 'steel' },
    { kind: 'add_support', id: 'test-left-support', targetNode: 'test-left', ux: true, uy: true, uz: true, rx: false, ry: false, rz: false },
    { kind: 'add_support', id: 'test-right-support', targetNode: 'test-right', ux: false, uy: true, uz: true, rx: false, ry: false, rz: false },
  ]);
});

test('opt-in connected conversation-agent qualification returns text and a typed proposal', {
  skip: process.env.FRAIA_LIVE_AGENT_QUALIFICATION !== '1'
    ? 'Set FRAIA_LIVE_AGENT_QUALIFICATION=1 with FRAIA_APPD_URL, FRAIA_APPD_TOKEN, and FRAIA_LIVE_AGENT_REQUEST_JSON.'
    : false,
}, async () => {
  const appdUrl = process.env.FRAIA_APPD_URL;
  const token = process.env.FRAIA_APPD_TOKEN;
  const payload = JSON.parse(process.env.FRAIA_LIVE_AGENT_REQUEST_JSON || 'null');
  assert.ok(appdUrl && token && payload, 'live qualification requires an active authenticated appd and exact request JSON');
  const started = Date.now();
  const response = await fetch(`${appdUrl.replace(/\/$/, '')}/conversations/agent/respond`, {
    method: 'POST',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
  const body = await response.json();
  assert.equal(response.ok, true, body.error || response.statusText);
  assert.ok(Date.now() - started >= 50, 'connected turn must exhibit observable work time');
  assert.ok(body.text?.trim().length >= 20, 'connected turn must return substantive conversational text');
  assert.ok(body.proposal?.operations?.length > 0, 'connected turn must return a typed proposal');
});
