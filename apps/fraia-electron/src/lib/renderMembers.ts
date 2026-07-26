import type { RenderMember, RenderNode, RenderScene } from './types';

export type DisplaySegment = { memberId: string; start: string; end: string; length: number };
export type DisplayMember = {
  id: string;
  role: string;
  nodeIds: string[];
  segments: DisplaySegment[];
  length: number;
  allowedSectionFamilies: string[];
  coordinationGroupLabels: string[];
  familyGroupLabels: string[];
  sectionCoordinationLabels: string[];
  sizeGroupLabels: string[];
  sizeCoordinationLabels: string[];
  schemeNotes: string[];
  sources: string[];
};

const eps = 1e-6;
function dist(a: RenderNode, b: RenderNode) {
  return Math.hypot(b.x - a.x, b.y - a.y, b.z - a.z);
}
function memberVector(member: RenderMember, nodes: Map<string, RenderNode>) {
  const start = nodes.get(member.start);
  const end = nodes.get(member.end);
  if (!start || !end) return null;
  return {
    x: end.x - start.x,
    y: end.y - start.y,
    z: end.z - start.z,
  };
}

function vectorLength(vector: { x: number; y: number; z: number }) {
  return Math.hypot(vector.x, vector.y, vector.z);
}

function vectorsAreCollinear(
  a: { x: number; y: number; z: number } | null,
  b: { x: number; y: number; z: number } | null,
) {
  if (!a || !b) return false;
  const aLength = vectorLength(a);
  const bLength = vectorLength(b);
  if (aLength <= eps || bLength <= eps) return false;
  const crossX = a.y * b.z - a.z * b.y;
  const crossY = a.z * b.x - a.x * b.z;
  const crossZ = a.x * b.y - a.y * b.x;
  return vectorLength({ x: crossX, y: crossY, z: crossZ }) / (aLength * bLength) <= eps;
}

function sharesNode(member: RenderMember, nodeId: string) {
  return member.start === nodeId || member.end === nodeId;
}

function otherNode(member: RenderMember, nodeId: string) {
  return member.start === nodeId ? member.end : member.start;
}

function collinearNeighbors(
  member: RenderMember,
  nodeId: string,
  members: RenderMember[],
  nodes: Map<string, RenderNode>,
  used = new Set<string>(),
) {
  const vector = memberVector(member, nodes);
  return members.filter((candidate) => (
    candidate.id !== member.id &&
    !used.has(candidate.id) &&
    sharesNode(candidate, nodeId) &&
    vectorsAreCollinear(vector, memberVector(candidate, nodes))
  ));
}

function preferredChainStart(member: RenderMember, members: RenderMember[], nodes: Map<string, RenderNode>) {
  const startNeighbors = collinearNeighbors(member, member.start, members, nodes);
  const endNeighbors = collinearNeighbors(member, member.end, members, nodes);
  if (startNeighbors.length === 0) return member.start;
  if (endNeighbors.length === 0) return member.end;
  return member.start;
}

function nextCollinearMember(
  previousMember: RenderMember,
  nodeId: string,
  members: RenderMember[],
  nodes: Map<string, RenderNode>,
  used: Set<string>,
) {
  return collinearNeighbors(previousMember, nodeId, members, nodes, used)[0] ?? null;
}

function memberChainFrom(
  seed: RenderMember,
  startId: string,
  members: RenderMember[],
  nodes: Map<string, RenderNode>,
  used: Set<string>,
) {
  const chain = [startId];
  const segs: DisplaySegment[] = [];
  let current = startId;
  let next: RenderMember | null = seed;
  while (next) {
    used.add(next.id);
    const other = otherNode(next, current);
    const a = nodes.get(current);
    const b = nodes.get(other);
    const length = a && b ? dist(a, b) : 0;
    segs.push({ memberId: next.id, start: current, end: other, length });
    chain.push(other);
    current = other;
    next = nextCollinearMember(next, current, members, nodes, used);
  }
  return { chain, segs };
}

export function displayMembersFor(scene: RenderScene): DisplayMember[] {
  const nodes = new Map(scene.nodes.map((n) => [n.id, n]));
  const buckets = new Map<string, RenderMember[]>();
  for (const m of scene.members) {
    const a = nodes.get(m.start), b = nodes.get(m.end);
    if (!a || !b) continue;
    buckets.set(m.role, [...(buckets.get(m.role) ?? []), m]);
  }

  const out: DisplayMember[] = [];
  let nextDisplayId = 1;
  for (const [role, members] of buckets) {
    const used = new Set<string>();
    for (const seed of members) {
      if (used.has(seed.id)) continue;
      const startId = preferredChainStart(seed, members, nodes);
      const { chain, segs } = memberChainFrom(seed, startId, members, nodes, used);
      const length = segs.reduce((sum, s) => sum + s.length, 0);
      const segmentMembers = segs
        .map((segment) => members.find((member) => member.id === segment.memberId))
        .filter(Boolean) as RenderMember[];
      const allowedSectionFamilies = uniqueStrings(segmentMembers.flatMap((member) => member.allowedSectionFamilies ?? []));
      const coordinationGroupLabels = uniqueStrings(segmentMembers.map((member) => member.coordinationGroupLabel).filter(Boolean) as string[]);
      const familyGroupLabels = uniqueStrings(segmentMembers.map((member) => member.familyGroupLabel).filter(Boolean) as string[]);
      const sectionCoordinationLabels = uniqueStrings(segmentMembers.map(sectionCoordinationLabel).filter(Boolean) as string[]);
      const sizeGroupLabels = uniqueStrings(segmentMembers.map((member) => member.sizeGroupLabel).filter(Boolean) as string[]);
      const sizeCoordinationLabels = uniqueStrings(segmentMembers.map(sizeCoordinationLabel).filter(Boolean) as string[]);
      const schemeNotes = uniqueStrings(segmentMembers.map((member) => member.schemeNote).filter(Boolean) as string[]);
      const sources = uniqueStrings(segmentMembers.map((member) => member.source).filter(Boolean) as string[]);
      out.push({ id: String(nextDisplayId), role, nodeIds: chain, segments: segs, length, allowedSectionFamilies, coordinationGroupLabels, familyGroupLabels, sectionCoordinationLabels, sizeGroupLabels, sizeCoordinationLabels, schemeNotes, sources });
      nextDisplayId += 1;
    }
  }
  return out;
}

export function memberLabel(member: DisplayMember) {
  const role = member.role === 'member' ? 'Member' : `${member.role.charAt(0).toUpperCase()}${member.role.slice(1)}`;
  return `${role} ${member.id}`;
}

function uniqueStrings(items: string[]) {
  return [...new Set(items.map((item) => item.trim()).filter(Boolean))];
}

function sectionCoordinationLabel(member: RenderMember) {
  const coordination = member.sectionCoordination;
  if (coordination?.kind === 'independent') return undefined;
  if (coordination?.kind === 'shared') return coordination.groupLabel ?? member.familyGroupLabel;
  return member.familyGroupLabel;
}

function sizeCoordinationLabel(member: RenderMember) {
  const coordination = member.sizeCoordination;
  if (coordination?.kind === 'independent') return undefined;
  if (coordination?.kind === 'shared') return coordination.groupLabel ?? member.sizeGroupLabel;
  return member.sizeGroupLabel;
}
