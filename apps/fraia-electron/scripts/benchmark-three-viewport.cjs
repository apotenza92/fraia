const { app, BrowserWindow } = require('electron');
const fs = require('node:fs');
const path = require('node:path');
const {
  frameBudgetForMembers,
  rendererWorkingSetBudgetForMembers,
  round,
  selectedBudget,
} = require('./perf-budgets.cjs');

function parseArgs(argv) {
  const config = {
    mode: 'batched',
    benchmark: 'random',
    members: 10000,
    labels: 'auto',
    frames: 120,
    warmup: 10,
    width: 1280,
    height: 820,
    maxAvgRenderMs: null,
    maxDrawCalls: null,
    maxRendererWorkingSetMb: null,
    output: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--mode') config.mode = argv[++index] ?? config.mode;
    else if (arg === '--benchmark' || arg === '--bench') config.benchmark = argv[++index] ?? config.benchmark;
    else if (arg === '--members') config.members = Number(argv[++index] ?? config.members);
    else if (arg === '--labels') config.labels = argv[++index] ?? config.labels;
    else if (arg === '--frames') config.frames = Number(argv[++index] ?? config.frames);
    else if (arg === '--warmup') config.warmup = Number(argv[++index] ?? config.warmup);
    else if (arg === '--width') config.width = Number(argv[++index] ?? config.width);
    else if (arg === '--height') config.height = Number(argv[++index] ?? config.height);
    else if (arg === '--max-avg-render-ms') config.maxAvgRenderMs = Number(argv[++index]);
    else if (arg === '--max-draw-calls') config.maxDrawCalls = Number(argv[++index]);
    else if (arg === '--max-renderer-working-set-mb') config.maxRendererWorkingSetMb = Number(argv[++index]);
    else if (arg === '--output') config.output = path.resolve(argv[++index] ?? '');
    else if (arg === '--help' || arg === '-h') {
      console.log(`Fraia Three.js viewport benchmark

Usage:
  npm run benchmark:viewport -- --mode object --benchmark random --members 10000 --labels off
  npm run benchmark:viewport -- --mode batched --benchmark multi --members 50000 --labels off
  npm run benchmark:viewport -- --mode batched --benchmark random --members 100000 --labels off

Options:
  --mode object|batched
  --benchmark grid|multi|random|portal
  --members <count>
  --labels on|off|auto
  --frames <count>
  --warmup <count>
  --max-avg-render-ms <ms>
  --max-draw-calls <count>
  --max-renderer-working-set-mb <mb>
  --output <path>`);
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (!['object', 'batched'].includes(config.mode)) {
    throw new Error(`Unknown benchmark mode: ${config.mode}`);
  }
  return config;
}

const config = parseArgs(process.argv.slice(2));
const performanceBudget = selectedBudget();
const electronRoot = path.resolve(__dirname, '..');
const defaultOutput = path.resolve(electronRoot, '..', '..', 'output', `viewport-benchmark-${config.mode}-${config.benchmark}-${config.members}.json`);
config.output ??= defaultOutput;

const benchmarkSource = String.raw`
async function runBenchmark(config, electronRoot, performanceBudget) {
  const THREE = require(electronRoot + '/node_modules/three');
  const { Line2 } = require(electronRoot + '/node_modules/three/examples/jsm/lines/Line2.js');
  const { LineGeometry } = require(electronRoot + '/node_modules/three/examples/jsm/lines/LineGeometry.js');
  const { LineMaterial } = require(electronRoot + '/node_modules/three/examples/jsm/lines/LineMaterial.js');

  function now() { return performance.now(); }
  function timed(fn) {
    const start = now();
    const value = fn();
    return { value, ms: now() - start };
  }
  function quantile(values, q) {
    if (!values.length) return null;
    const sorted = [...values].sort((a, b) => a - b);
    return sorted[Math.min(sorted.length - 1, Math.max(0, Math.floor((sorted.length - 1) * q)))];
  }
  function finiteAverage(values) {
    const finite = values.filter(Number.isFinite);
    return finite.length ? finite.reduce((sum, value) => sum + value, 0) / finite.length : null;
  }

  async function processMemorySnapshot() {
    const info = typeof process.getProcessMemoryInfo === 'function'
      ? await process.getProcessMemoryInfo()
      : null;
    const heap = performance.memory
      ? {
          usedJsHeapSizeMb: performance.memory.usedJSHeapSize / 1024 / 1024,
          totalJsHeapSizeMb: performance.memory.totalJSHeapSize / 1024 / 1024,
          jsHeapSizeLimitMb: performance.memory.jsHeapSizeLimit / 1024 / 1024,
        }
      : null;
    return {
      workingSetMb: info ? info.workingSetSize / 1024 : null,
      peakWorkingSetMb: info ? info.peakWorkingSetSize / 1024 : null,
      privateBytesMb: info ? info.privateBytes / 1024 : null,
      sharedBytesMb: info ? info.sharedBytes / 1024 : null,
      heap,
    };
  }

  function generateScene(kind, targetMembers) {
    if (kind === 'grid' || kind === 'grid-frame') return generateGridFrame(targetMembers);
    if (kind === 'multi' || kind === 'multi-storey') return generateMultiStoreyFrame(targetMembers);
    if (kind === 'portal' || kind === 'repeated-portal') return generateRepeatedPortal(targetMembers);
    return generateRandomTruss(targetMembers);
  }

  function generateGridFrame(targetMembers) {
    const cols = Math.max(2, Math.ceil(Math.sqrt(targetMembers)));
    const rows = Math.max(2, Math.max(1, Math.floor(targetMembers / cols)) + 1);
    const nodes = [];
    for (let y = 0; y < rows; y += 1) {
      for (let x = 0; x < cols; x += 1) nodes.push({ id: 'N' + (y * cols + x + 1), x: x * 3, y: y * 3, z: 0 });
    }
    const members = [];
    for (let y = 0; y < rows; y += 1) {
      for (let x = 0; x < cols - 1 && members.length < targetMembers; x += 1) {
        members.push({ id: 'M' + (members.length + 1), start: 'N' + (y * cols + x + 1), end: 'N' + (y * cols + x + 2), role: 'beam' });
      }
    }
    for (let y = 0; y < rows - 1; y += 1) {
      for (let x = 0; x < cols && members.length < targetMembers; x += 1) {
        members.push({ id: 'M' + (members.length + 1), start: 'N' + (y * cols + x + 1), end: 'N' + ((y + 1) * cols + x + 1), role: 'column' });
      }
    }
    return { nodes, members, supports: [], loads: [], releases: [] };
  }

  function node3dId(x, y, z, nx, nz) { return 'N' + (y * nx * nz + z * nx + x + 1); }
  function pushMember3d(members, limit, ax, ay, az, bx, by, bz, nx, nz, role) {
    if (members.length >= limit) return;
    members.push({ id: 'M' + (members.length + 1), start: node3dId(ax, ay, az, nx, nz), end: node3dId(bx, by, bz, nx, nz), role });
  }
  function generateMultiStoreyFrame(targetMembers) {
    const baysX = Math.max(2, Math.ceil(Math.sqrt(targetMembers / 6)));
    const baysZ = Math.max(2, Math.floor(baysX / 2));
    const storeys = Math.min(40, Math.max(2, Math.floor(targetMembers / (baysX * baysZ * 3))));
    const nx = baysX + 1, nz = baysZ + 1, ny = storeys + 1;
    const nodes = [];
    for (let y = 0; y < ny; y += 1) for (let z = 0; z < nz; z += 1) for (let x = 0; x < nx; x += 1) {
      nodes.push({ id: node3dId(x, y, z, nx, nz), x: x * 7.5, y: y * 3.6, z: z * 7.5 });
    }
    const members = [];
    for (let y = 0; y < ny; y += 1) {
      for (let z = 0; z < nz; z += 1) for (let x = 0; x < baysX; x += 1) pushMember3d(members, targetMembers, x, y, z, x + 1, y, z, nx, nz, 'beam');
      for (let z = 0; z < baysZ; z += 1) for (let x = 0; x < nx; x += 1) pushMember3d(members, targetMembers, x, y, z, x, y, z + 1, nx, nz, 'beam');
    }
    for (let y = 0; y < storeys; y += 1) for (let z = 0; z < nz; z += 1) for (let x = 0; x < nx; x += 1) {
      pushMember3d(members, targetMembers, x, y, z, x, y + 1, z, nx, nz, 'column');
    }
    return { nodes, members, supports: [], loads: [], releases: [] };
  }

  function lcg(seedBox) {
    seedBox.seed = (BigInt.asUintN(64, seedBox.seed * 6364136223846793005n + 1n));
    return Number(seedBox.seed >> 32n) / 4294967295;
  }
  function generateRandomTruss(targetMembers) {
    const nodeCount = Math.max(8, Math.floor(targetMembers / 2));
    const seedBox = { seed: 0x5f3759dfn };
    const nodes = [];
    for (let index = 0; index < nodeCount; index += 1) {
      nodes.push({ id: 'N' + (index + 1), x: lcg(seedBox) * 180 - 90, y: lcg(seedBox) * 72, z: lcg(seedBox) * 180 - 90 });
    }
    const members = [];
    for (let index = 0; index < targetMembers; index += 1) {
      const a = index % nodeCount;
      const stride = 1 + Math.floor(lcg(seedBox) * Math.max(1, nodeCount / 3));
      const b = Math.min(a + stride, nodeCount - 1);
      if (a !== b) members.push({ id: 'M' + (members.length + 1), start: 'N' + (a + 1), end: 'N' + (b + 1), role: 'brace' });
    }
    return { nodes, members, supports: [], loads: [], releases: [] };
  }

  function generateRepeatedPortal(targetMembers) {
    const portals = Math.max(1, Math.ceil(targetMembers / 5));
    const nodes = [], members = [];
    for (let bay = 0; bay < portals; bay += 1) {
      const x = bay * 6, base = bay * 4;
      nodes.push({ id: 'N' + (base + 1), x, y: 0, z: 0 }, { id: 'N' + (base + 2), x, y: 6, z: 0 }, { id: 'N' + (base + 3), x, y: 6, z: 12 }, { id: 'N' + (base + 4), x, y: 0, z: 12 });
      for (const [a, b, role] of [[1, 2, 'column'], [2, 3, 'rafter'], [3, 4, 'column']]) {
        if (members.length < targetMembers) members.push({ id: 'M' + (members.length + 1), start: 'N' + (base + a), end: 'N' + (base + b), role });
      }
      if (bay > 0) for (const local of [1, 2, 3, 4]) {
        if (members.length < targetMembers) members.push({ id: 'M' + (members.length + 1), start: 'N' + (base + local - 4), end: 'N' + (base + local), role: 'purlin' });
      }
    }
    return { nodes, members, supports: [], loads: [], releases: [] };
  }

  function dist(a, b) { return Math.hypot(b.x - a.x, b.y - a.y, b.z - a.z); }
  function axisKey(a, b) {
    const dx = Math.abs(b.x - a.x), dy = Math.abs(b.y - a.y), dz = Math.abs(b.z - a.z);
    if (dx >= dy && dx >= dz) return 'x:' + a.y.toFixed(6) + ':' + a.z.toFixed(6);
    if (dy >= dx && dy >= dz) return 'y:' + a.x.toFixed(6) + ':' + a.z.toFixed(6);
    return 'z:' + a.x.toFixed(6) + ':' + a.y.toFixed(6);
  }
  function coordForKey(key, n) { return key.startsWith('x:') ? n.x : key.startsWith('y:') ? n.y : n.z; }
  function displayMembersFor(scene) {
    const nodes = new Map(scene.nodes.map((n) => [n.id, n]));
    const buckets = new Map();
    for (const m of scene.members) {
      const a = nodes.get(m.start), b = nodes.get(m.end);
      if (!a || !b) continue;
      const key = (m.role || 'member') + ':' + axisKey(a, b);
      const bucket = buckets.get(key) || [];
      bucket.push(m);
      buckets.set(key, bucket);
    }
    const out = [];
    for (const [key, members] of buckets) {
      const role = key.split(':')[0] || 'member';
      const degree = new Map();
      for (const m of members) {
        degree.set(m.start, (degree.get(m.start) || 0) + 1);
        degree.set(m.end, (degree.get(m.end) || 0) + 1);
      }
      const used = new Set();
      const starts = [...degree.entries()].filter(([, d]) => d === 1).map(([id]) => id);
      const candidates = [...starts, ...members.flatMap((m) => [m.start, m.end])];
      for (const startId of candidates) {
        const pending = members.filter((m) => !used.has(m.id) && (m.start === startId || m.end === startId));
        if (!pending.length) continue;
        const chain = [startId], segs = [];
        let current = startId;
        while (true) {
          const next = members.find((m) => !used.has(m.id) && (m.start === current || m.end === current));
          if (!next) break;
          used.add(next.id);
          const other = next.start === current ? next.end : next.start;
          const a = nodes.get(current), b = nodes.get(other);
          segs.push({ memberId: next.id, start: current, end: other, length: a && b ? dist(a, b) : 0 });
          chain.push(other);
          current = other;
        }
        const sortedIds = [...chain].sort((a, b) => coordForKey(key, nodes.get(a)) - coordForKey(key, nodes.get(b)));
        out.push({ id: String(out.length + 1), role, nodeIds: sortedIds.length === chain.length ? sortedIds : chain, segments: segs, length: segs.reduce((sum, s) => sum + s.length, 0) });
      }
    }
    return out;
  }

  function makeLabelTexture(text) {
    const canvas = document.createElement('canvas');
    canvas.width = 160;
    canvas.height = 44;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#111827';
    ctx.font = '800 12px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(text, canvas.width / 2, canvas.height / 2);
    return new THREE.CanvasTexture(canvas);
  }

  function pointToVector(node) {
    return new THREE.Vector3(node.x, node.y, node.z);
  }

  function projectToViewport(camera, width, height, point) {
    const projected = point.clone().project(camera);
    return {
      x: (projected.x * 0.5 + 0.5) * width,
      y: (-projected.y * 0.5 + 0.5) * height,
      z: projected.z,
    };
  }

  function distanceToScreenSegment(point, start, end) {
    const dx = end.x - start.x, dy = end.y - start.y;
    const lengthSquared = dx * dx + dy * dy;
    if (lengthSquared <= 1e-8) return Math.hypot(point.x - start.x, point.y - start.y);
    const t = Math.max(0, Math.min(1, ((point.x - start.x) * dx + (point.y - start.y) * dy) / lengthSquared));
    return Math.hypot(point.x - (start.x + dx * t), point.y - (start.y + dy * t));
  }

  function buildHitIndex(scene, nodesById, camera, width, height) {
    camera.updateMatrixWorld(true);
    const out = [];
    for (const member of scene.members) {
      const a = nodesById.get(member.start), b = nodesById.get(member.end);
      if (!a || !b) continue;
      const start = projectToViewport(camera, width, height, pointToVector(a));
      const end = projectToViewport(camera, width, height, pointToVector(b));
      if ((start.z < -1 || start.z > 1) && (end.z < -1 || end.z > 1)) continue;
      out.push({ id: member.id, start, end });
    }
    return out;
  }

  function runHitTests(hitIndex, width, height) {
    const tests = [];
    const count = Math.min(48, Math.max(12, Math.floor(hitIndex.length / 2000)));
    for (let index = 0; index < count; index += 1) {
      const base = hitIndex[(index * 7919) % Math.max(1, hitIndex.length)];
      tests.push(base ? { x: (base.start.x + base.end.x) / 2 + (index % 5) - 2, y: (base.start.y + base.end.y) / 2 } : { x: width / 2, y: height / 2 });
    }
    const times = [];
    let hits = 0;
    for (const test of tests) {
      const started = now();
      let best = null;
      for (const segment of hitIndex) {
        const distance = distanceToScreenSegment(test, segment.start, segment.end);
        if (!best || distance < best.distance) best = { id: segment.id, distance };
      }
      if (best && best.distance <= 14) hits += 1;
      times.push(now() - started);
    }
    return {
      count: tests.length,
      hits,
      avgMs: finiteAverage(times),
      p95Ms: quantile(times, 0.95),
      maxMs: times.length ? Math.max(...times) : null,
    };
  }

  function fitCamera(camera, nodes) {
    const box = new THREE.Box3();
    for (const node of nodes) box.expandByPoint(pointToVector(node));
    const centre = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    const radius = Math.max(size.x, size.y, size.z, 1);
    camera.position.copy(centre).add(new THREE.Vector3(radius * 0.9, radius * 0.65, radius * 0.9));
    camera.lookAt(centre);
    camera.left = -radius * 0.65;
    camera.right = radius * 0.65;
    camera.top = radius * 0.42;
    camera.bottom = -radius * 0.42;
    camera.near = -radius * 10;
    camera.far = radius * 10;
    camera.updateProjectionMatrix();
    camera.updateMatrixWorld(true);
  }

  function addNodeBatch(THREE, threeScene, scene, pointMat) {
    const positions = new Float32Array(scene.nodes.length * 3);
    for (let index = 0; index < scene.nodes.length; index += 1) {
      const node = scene.nodes[index], offset = index * 3;
      positions[offset] = node.x;
      positions[offset + 1] = node.y;
      positions[offset + 2] = node.z;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    threeScene.add(new THREE.Points(geo, pointMat));
  }

  function buildBatchedScene(THREE, threeScene, scene, nodesById, labelsEnabled, textures, materials) {
    const positions = new Float32Array(scene.members.length * 6);
    let cursor = 0;
    for (const member of scene.members) {
      const a = nodesById.get(member.start), b = nodesById.get(member.end);
      if (!a || !b) continue;
      positions[cursor++] = a.x; positions[cursor++] = a.y; positions[cursor++] = a.z;
      positions[cursor++] = b.x; positions[cursor++] = b.y; positions[cursor++] = b.z;
    }
    const memberGeo = new THREE.BufferGeometry();
    memberGeo.setAttribute('position', new THREE.BufferAttribute(positions.slice(0, cursor), 3));
    threeScene.add(new THREE.LineSegments(memberGeo, materials.memberBasic));
    addNodeBatch(THREE, threeScene, scene, materials.pointMat);
    let labelCount = 0;
    if (labelsEnabled) {
      for (let index = 0; index < Math.min(scene.members.length, 10000); index += 1) {
        const member = scene.members[index];
        const a = nodesById.get(member.start);
        if (!a) continue;
        const texture = makeLabelTexture('Member ' + member.id);
        textures.push(texture);
        const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true }));
        sprite.position.set(a.x, a.y, a.z);
        threeScene.add(sprite);
        labelCount += 1;
      }
    }
    return { memberLineObjectCount: 1, labelCount };
  }

  function buildObjectScene(THREE, threeScene, scene, nodesById, displayMembers, labelsEnabled, textures, materials) {
    let memberLineObjectCount = 0, labelCount = 0;
    for (const member of displayMembers) {
      const positions = [];
      for (const id of member.nodeIds) {
        const node = nodesById.get(id);
        if (node) positions.push(node.x, node.y, node.z);
      }
      if (positions.length < 6) continue;
      const geo = new LineGeometry();
      geo.setPositions(positions);
      const line = new Line2(geo, materials.memberLine);
      line.computeLineDistances();
      threeScene.add(line);
      memberLineObjectCount += 1;
      if (labelsEnabled) {
        const texture = makeLabelTexture('Member ' + member.id);
        textures.push(texture);
        const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture, depthTest: false, depthWrite: false, transparent: true }));
        const first = nodesById.get(member.nodeIds[0]);
        if (first) sprite.position.set(first.x, first.y, first.z);
        threeScene.add(sprite);
        labelCount += 1;
      }
    }
    addNodeBatch(THREE, threeScene, scene, materials.pointMat);
    return { memberLineObjectCount, labelCount };
  }

  const labelsEnabled = config.labels === 'on' || (config.labels === 'auto' && config.members < 10000);
  const metrics = {
    mode: config.mode,
    benchmark: config.benchmark,
    requestedMembers: config.members,
    labels: labelsEnabled ? 'on' : 'off',
    performanceBudget,
  };
  metrics.memoryBeforeSceneMb = await processMemorySnapshot();
  const generated = timed(() => generateScene(config.benchmark, config.members));
  const scene = generated.value;
  metrics.sceneGenerationMs = generated.ms;
  metrics.nodeCount = scene.nodes.length;
  metrics.memberCount = scene.members.length;

  const displayTimed = timed(() => config.mode === 'object' ? displayMembersFor(scene) : []);
  const displayMembers = displayTimed.value;
  metrics.rendererPrepMs = displayTimed.ms;
  metrics.displayMemberCount = displayMembers.length;

  const root = document.createElement('div');
  root.style.width = config.width + 'px';
  root.style.height = config.height + 'px';
  document.body.appendChild(root);
  const renderer = new THREE.WebGLRenderer({ antialias: config.members <= 50000, alpha: false, powerPreference: 'high-performance' });
  renderer.setPixelRatio(1);
  renderer.setSize(config.width, config.height);
  root.appendChild(renderer.domElement);
  const threeScene = new THREE.Scene();
  threeScene.background = new THREE.Color('#050b14');
  const camera = new THREE.OrthographicCamera(-100, 100, 100, -100, -100000, 100000);
  fitCamera(camera, scene.nodes);

  const nodesById = new Map(scene.nodes.map((n) => [n.id, n]));
  const materials = {
    memberLine: new LineMaterial({ linewidth: 2, worldUnits: false, color: 0xd1d5db }),
    memberBasic: new THREE.LineBasicMaterial({ color: 0xd1d5db, depthTest: true, depthWrite: false }),
    pointMat: new THREE.PointsMaterial({ size: config.members >= 100000 ? 3 : 6, sizeAttenuation: false, color: 0x60a5fa }),
  };
  materials.memberLine.resolution.set(config.width, config.height);
  const textures = [];

  const buildTimed = timed(() => config.mode === 'object'
    ? buildObjectScene(THREE, threeScene, scene, nodesById, displayMembers, labelsEnabled, textures, materials)
    : buildBatchedScene(THREE, threeScene, scene, nodesById, labelsEnabled, textures, materials));
  metrics.buildThreeSceneMs = buildTimed.ms;
  metrics.bufferUploadPrepMs = buildTimed.ms;
  metrics.memberLineObjectCount = buildTimed.value.memberLineObjectCount;
  metrics.labelCount = buildTimed.value.labelCount;
  metrics.textureCount = textures.length;
  metrics.threeObjectCount = threeScene.children.length;
  metrics.memoryAfterSceneBuildMb = await processMemorySnapshot();

  const hitIndexTimed = timed(() => buildHitIndex(scene, nodesById, camera, config.width, config.height));
  metrics.hitIndexBuildMs = hitIndexTimed.ms;
  metrics.hitIndexSegmentCount = hitIndexTimed.value.length;
  metrics.hitTest = runHitTests(hitIndexTimed.value, config.width, config.height);

  const gl = renderer.getContext();
  const firstUploadStart = now();
  renderer.render(threeScene, camera);
  if (gl.finish) gl.finish();
  metrics.bufferUploadMs = now() - firstUploadStart;
  for (let index = 0; index < config.warmup; index += 1) {
    renderer.render(threeScene, camera);
    if (gl.finish) gl.finish();
  }
  const renderTimes = [];
  for (let index = 0; index < config.frames; index += 1) {
    const started = now();
    renderer.render(threeScene, camera);
    if (gl.finish) gl.finish();
    renderTimes.push(now() - started);
  }
  metrics.frames = config.frames;
  metrics.warmupFrames = config.warmup;
  metrics.avgRenderMs = finiteAverage(renderTimes);
  metrics.p95RenderMs = quantile(renderTimes, 0.95);
  metrics.maxRenderMs = renderTimes.length ? Math.max(...renderTimes) : null;
  metrics.drawCalls = renderer.info.render.calls;
  metrics.rendererInfo = {
    calls: renderer.info.render.calls,
    triangles: renderer.info.render.triangles,
    points: renderer.info.render.points,
    lines: renderer.info.render.lines,
    geometries: renderer.info.memory.geometries,
    textures: renderer.info.memory.textures,
  };
  metrics.memoryAfterRenderMb = await processMemorySnapshot();
  renderer.dispose();
  return metrics;
}
`;

function appMetricSummary(metric) {
  return {
    type: metric.type,
    pid: metric.pid,
    cpuPercent: metric.cpu?.percentCPUUsage ?? null,
    workingSetMb: metric.memory?.workingSetSize != null ? metric.memory.workingSetSize / 1024 : null,
    peakWorkingSetMb: metric.memory?.peakWorkingSetSize != null ? metric.memory.peakWorkingSetSize / 1024 : null,
  };
}

function rendererWorkingSetFromAppMetrics(appMetrics) {
  const rendererMetric = (appMetrics ?? []).find((metric) => metric.type === 'Tab' || metric.type === 'Renderer');
  return rendererMetric?.workingSetMb ?? null;
}

function drawCallBudgetForMembers(memberCount) {
  if (memberCount <= 10000) return 40;
  if (memberCount <= 50000) return 60;
  return 80;
}

async function main() {
  await app.whenReady();
  const win = new BrowserWindow({
    show: false,
    width: config.width,
    height: config.height,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      backgroundThrottling: false,
    },
  });
  await win.loadURL('data:text/html,<html><body style="margin:0;background:#050b14"></body></html>');
  const result = await win.webContents.executeJavaScript(
    `(${benchmarkSource})(${JSON.stringify(config)}, ${JSON.stringify(electronRoot)}, ${JSON.stringify(performanceBudget)})`,
    true,
  );
  result.appMetrics = app.getAppMetrics().map(appMetricSummary);
  result.rendererWorkingSetMb = rendererWorkingSetFromAppMetrics(result.appMetrics) ?? result.memoryAfterRenderMb?.workingSetMb ?? null;

  const effectiveMaxAvgRenderMs = config.maxAvgRenderMs ?? (config.mode === 'batched' ? frameBudgetForMembers(performanceBudget, result.memberCount) : null);
  const effectiveMaxDrawCalls = config.maxDrawCalls ?? (config.mode === 'batched' ? drawCallBudgetForMembers(result.memberCount) : null);
  const effectiveMaxRendererWorkingSetMb = config.maxRendererWorkingSetMb ?? (config.mode === 'batched' ? rendererWorkingSetBudgetForMembers(performanceBudget, result.memberCount) : null);
  result.effectiveBudgets = {
    maxAvgRenderMs: effectiveMaxAvgRenderMs,
    maxDrawCalls: effectiveMaxDrawCalls,
    maxRendererWorkingSetMb: effectiveMaxRendererWorkingSetMb,
  };
  result.output = config.output;
  fs.mkdirSync(path.dirname(config.output), { recursive: true });
  fs.writeFileSync(config.output, `${JSON.stringify(result, null, 2)}\n`);

  console.log(JSON.stringify(result, null, 2));

  const failures = [];
  if (Number.isFinite(effectiveMaxAvgRenderMs) && result.avgRenderMs > effectiveMaxAvgRenderMs) {
    failures.push(`Average render time ${round(result.avgRenderMs, 2)} ms exceeded budget ${effectiveMaxAvgRenderMs} ms.`);
  }
  if (Number.isFinite(effectiveMaxDrawCalls) && result.drawCalls > effectiveMaxDrawCalls) {
    failures.push(`Draw calls ${result.drawCalls} exceeded budget ${effectiveMaxDrawCalls}.`);
  }
  if (Number.isFinite(effectiveMaxRendererWorkingSetMb) && result.rendererWorkingSetMb > effectiveMaxRendererWorkingSetMb) {
    failures.push(`Renderer working set ${round(result.rendererWorkingSetMb)} MB exceeded budget ${effectiveMaxRendererWorkingSetMb} MB.`);
  }
  if (config.mode === 'batched' && result.memberCount <= 50000 && Number.isFinite(result.hitTest?.p95Ms) && result.hitTest.p95Ms > 16) {
    failures.push(`Hit-test p95 ${round(result.hitTest.p95Ms, 2)} ms exceeded budget 16 ms.`);
  }

  win.close();
  app.quit();
  if (failures.length) {
    throw new Error(failures.join('\n'));
  }
}

main().catch((error) => {
  console.error(error);
  app.exit(1);
});
