export type ElementLabelKind = 'node' | 'member' | 'support' | 'load';
export type GroupLabelKind = 'family' | 'designation' | 'support';

const elementPrefixes: Record<ElementLabelKind, string> = {
  node: 'N',
  member: 'M',
  support: 'S',
  load: 'L',
};

const groupPrefixes: Record<GroupLabelKind, string> = {
  family: 'GF',
  designation: 'GD',
  support: 'GS',
};

export function formatElementLabel(kind: ElementLabelKind, value: string | number) {
  const raw = String(value).trim();
  const prefix = elementPrefixes[kind];
  if (new RegExp(`^${prefix}\\d`, 'i').test(raw)) return raw.toUpperCase();
  return `${prefix}${raw}`;
}

export function compactGroupOrdinal(label: string) {
  const trimmed = label.trim();
  const withoutName = trimmed
    .replace(/^Family Group\s+/i, '')
    .replace(/^Size Group\s+/i, '')
    .replace(/^Designation Group\s+/i, '')
    .replace(/^Support Group\s+/i, '');
  return withoutName.replace(/^(GF|GD|GS)\s*/i, '').trim();
}

export function formatGroupLabel(kind: GroupLabelKind, label: string) {
  const prefix = groupPrefixes[kind];
  const ordinal = compactGroupOrdinal(label);
  if (!ordinal) return prefix;
  if (ordinal.toUpperCase().startsWith(prefix)) return ordinal.toUpperCase();
  return `${prefix}${ordinal}`;
}

export function formatFamilyGroupLabel(label: string) {
  return formatGroupLabel('family', label);
}

export function formatDesignationGroupLabel(label: string) {
  return formatGroupLabel('designation', label);
}

export function formatSupportGroupLabel(label: string) {
  return formatGroupLabel('support', label);
}

export function formatReasoningEffortLabel(effort: string) {
  const normalized = effort.trim().replace(/[-_]+/g, ' ');
  if (!normalized) return effort;
  return `${normalized.charAt(0).toUpperCase()}${normalized.slice(1).toLowerCase()}`;
}

export function isSupportGroupLabel(label: string) {
  return /^(Support Group\s+\d+|GS\d+)/i.test(label.trim());
}

function uniqueLabels(labels: string[]) {
  return [...new Set(labels.map((label) => label.trim()).filter(Boolean))];
}

export function formatMemberGroupContext(familyLabels: string[], designationLabels: string[]) {
  const families = uniqueLabels(familyLabels).map(formatFamilyGroupLabel);
  const designations = uniqueLabels(designationLabels).map(formatDesignationGroupLabel);
  return [...families, ...designations].join(' : ');
}
