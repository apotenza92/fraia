import { useMemo } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import type { RenderScene, RenderSupport } from '../../lib/types';
import { displayMembersFor } from '../../lib/renderMembers';
import { compactGroupOrdinal, formatDesignationGroupLabel, formatElementLabel, formatFamilyGroupLabel, formatSupportGroupLabel, isSupportGroupLabel } from '../../lib/displayLabels';

type SchemeGroupsRow = {
  type: 'Family' | 'Designation' | 'Support';
  id: string;
  descriptionLines: string[];
  elementLabel: 'Members' | 'Supports';
  elementIds: string;
};

function isBriefVisualSupport(support: RenderSupport) {
  return support.id.startsWith('brief-visual-support-');
}

function supportType(support: RenderSupport) {
  if (isBriefVisualSupport(support)) return 'Indicative';
  const translations = [support.ux, support.uy, support.uz].filter(Boolean).length;
  const rotations = [support.rx, support.ry, support.rz].filter(Boolean).length;
  if ([support.ux, support.uy, support.uz, support.rx, support.ry, support.rz].every(Boolean)) return 'Fixed';
  if (translations >= 2 && rotations === 0) return 'Pinned';
  if (translations === 1 && rotations === 0) return 'Roller';
  return 'Support';
}

function supportGroupLabel(support: RenderSupport) {
  return support.supportGroupLabel ?? support.support_group_label;
}

function isSizeIndependentLabel(label: string) {
  return ['size independent', 'unique'].includes(label.trim().toLowerCase());
}

function memberSizeCoordination(member: { sizeCoordination?: { kind?: string; groupLabel?: string; group_label?: string }; sizeGroupLabel?: string }) {
  const kind = member.sizeCoordination?.kind?.toLowerCase();
  if (kind === 'independent') return { kind: 'independent' as const };
  if (kind === 'shared') return { kind: 'shared' as const, groupLabel: member.sizeCoordination?.groupLabel ?? member.sizeCoordination?.group_label ?? member.sizeGroupLabel };
  if (member.sizeGroupLabel) {
    return isSizeIndependentLabel(member.sizeGroupLabel)
      ? { kind: 'independent' as const }
      : { kind: 'shared' as const, groupLabel: member.sizeGroupLabel };
  }
  return { kind: 'unspecified' as const };
}

function memberSectionCoordination(member: { sectionCoordination?: { kind?: string; groupLabel?: string; group_label?: string }; familyGroupLabel?: string }) {
  const kind = member.sectionCoordination?.kind?.toLowerCase();
  if (kind === 'independent') return { kind: 'independent' as const };
  if (kind === 'shared') return { kind: 'shared' as const, groupLabel: member.sectionCoordination?.groupLabel ?? member.sectionCoordination?.group_label ?? member.familyGroupLabel };
  if (member.familyGroupLabel) return { kind: 'shared' as const, groupLabel: member.familyGroupLabel };
  return { kind: 'unspecified' as const };
}

function formatInstances(labels: string[]) {
  return labels.join(', ');
}

function sentenceCase(text: string) {
  const trimmed = text.trim();
  if (!trimmed) return trimmed;
  return `${trimmed.charAt(0).toUpperCase()}${trimmed.slice(1)}`;
}

function formatOrList(items: string[]) {
  if (items.length <= 2) return items.join(' or ');
  return `${items.slice(0, -1).join(', ')}, or ${items[items.length - 1]}`;
}

function familyLockDescriptionLines(options: string[]) {
  return [
    'Same section family:',
    options.length ? formatOrList(options) : 'Coordinated family set',
  ];
}

function schemeGroupTypeDescription(type: SchemeGroupsRow['type']) {
  if (type === 'Designation') return 'Designation group';
  if (type === 'Support') return 'Support group';
  return 'Family group';
}

function schemeGroupsRows(scene: RenderScene): SchemeGroupsRow[] {
  const memberInstances = new Map<string, string>();
  displayMembersFor(scene).forEach((member) => {
    member.segments.forEach((segment) => memberInstances.set(segment.memberId, formatElementLabel('member', member.id)));
  });
  const supportInstances = new Map<string, string>();
  (scene.supports ?? []).forEach((support, index) => supportInstances.set(support.id, formatElementLabel('support', index + 1)));
  const sectionGroups = new Map<string, { options: Set<string>; members: Set<string>; sizeGroups: Map<string, Set<string>> }>();
  const supportGroups = new Map<string, { types: Set<string>; supports: Set<string> }>();

  scene.members.forEach((member) => {
    const section = memberSectionCoordination(member);
    if (section.kind === 'shared' && section.groupLabel) {
      const group = sectionGroups.get(section.groupLabel) ?? { options: new Set<string>(), members: new Set<string>(), sizeGroups: new Map<string, Set<string>>() };
      (member.allowedSectionFamilies ?? []).forEach((family) => group.options.add(family));
      group.members.add(member.id);
      const size = memberSizeCoordination(member);
      const sizeKey = size.kind === 'independent' ? undefined : size.groupLabel;
      if (sizeKey) {
        const sizeMembers = group.sizeGroups.get(sizeKey) ?? new Set<string>();
        sizeMembers.add(member.id);
        group.sizeGroups.set(sizeKey, sizeMembers);
      }
      sectionGroups.set(section.groupLabel, group);
    }
  });

  (scene.supports ?? []).forEach((support) => {
    if (isBriefVisualSupport(support)) return;
    const label = supportGroupLabel(support);
    if (!label || !isSupportGroupLabel(label)) return;
    const group = supportGroups.get(label) ?? { types: new Set<string>(), supports: new Set<string>() };
    group.types.add(supportType(support));
    group.supports.add(support.id);
    supportGroups.set(label, group);
  });

  const rows: SchemeGroupsRow[] = [];
  [...sectionGroups.entries()]
    .sort(([a], [b]) => compactGroupOrdinal(a).localeCompare(compactGroupOrdinal(b), undefined, { numeric: true }))
    .forEach(([label, group]) => {
      if (group.members.size <= 1) return;
      const options = [...group.options];
      const sectionId = formatFamilyGroupLabel(label);
      const instances = [...group.members].map((id) => memberInstances.get(id) ?? formatElementLabel('member', id));
      rows.push({
        type: 'Family',
        id: sectionId,
        descriptionLines: familyLockDescriptionLines(options),
        elementLabel: 'Members',
        elementIds: formatInstances(instances),
      });
      [...group.sizeGroups.entries()]
        .sort(([a], [b]) => compactGroupOrdinal(a).localeCompare(compactGroupOrdinal(b), undefined, { numeric: true }))
        .forEach(([sizeLabel, members]) => {
          if (members.size <= 1) return;
          const sizeInstances = [...members].map((id) => memberInstances.get(id) ?? formatElementLabel('member', id));
          rows.push({
            type: 'Designation',
            id: formatDesignationGroupLabel(sizeLabel),
            descriptionLines: ['Designation locked within:', sectionId],
            elementLabel: 'Members',
            elementIds: formatInstances(sizeInstances),
          });
        });
    });

  [...supportGroups.entries()]
    .sort(([a], [b]) => compactGroupOrdinal(a).localeCompare(compactGroupOrdinal(b), undefined, { numeric: true }))
    .forEach(([label, group]) => {
      if (group.supports.size <= 1) return;
      const instances = [...group.supports].map((id) => supportInstances.get(id) ?? formatElementLabel('support', id));
      rows.push({
        type: 'Support',
        id: formatSupportGroupLabel(label),
        descriptionLines: ['Support type:', sentenceCase([...group.types].join(' or '))],
        elementLabel: 'Supports',
        elementIds: formatInstances(instances),
      });
    });

  return rows;
}

export function sceneHasSchemeGroups(scene: RenderScene) {
  return schemeGroupsRows(scene).length > 0;
}

export function SchemeGroupsPanelContent({ scene }: { scene: RenderScene }) {
  const rows = useMemo(() => schemeGroupsRows(scene), [scene]);
  if (!rows.length) return null;

  return (
    <div className="h-full overflow-auto p-2">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Group ID</TableHead>
            <TableHead>Element ID</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((row) => (
            <TableRow key={`${row.type}-${row.id}-${row.descriptionLines.join('|')}`}>
              <TableCell>
                <span className="font-semibold">{row.id}</span>
              </TableCell>
              <TableCell>
                <div className="flex flex-col gap-0">
                  <span className="text-sm text-muted-foreground">{row.elementLabel}:</span>
                  <span className="text-sm font-medium">{row.elementIds}</span>
                </div>
              </TableCell>
              <TableCell>
                <div className="flex flex-col gap-0">
                  <span className="text-sm text-muted-foreground">Type: {schemeGroupTypeDescription(row.type)}</span>
                  {row.descriptionLines.map((line) => (
                    <span key={line} className="text-sm font-medium">{line}</span>
                  ))}
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

export function SchemeGroupsOverlay({ scene }: { scene: RenderScene }) {
  return (
    <aside className="absolute right-2 top-2 max-h-[min(460px,calc(100%-1rem))] w-[min(420px,calc(100%-1rem))] overflow-hidden rounded-xl border bg-card text-card-foreground shadow-xl">
      <SchemeGroupsPanelContent scene={scene} />
    </aside>
  );
}
