import type { UnitProfile, QuantityKind } from './types';

export const metricStructuralUnitProfile: UnitProfile = {
  id: 'metric_structural',
  label: 'Metric structural',
  system: 'metric_structural',
  length: { symbol: 'mm', canonicalToDisplay: 1000, precision: 0 },
  force: { symbol: 'kN', canonicalToDisplay: 0.001, precision: 3 },
  lineLoad: { symbol: 'kN/m', canonicalToDisplay: 0.001, precision: 3 },
  moment: { symbol: 'kN*m', canonicalToDisplay: 0.001, precision: 3 },
  stress: { symbol: 'MPa', canonicalToDisplay: 0.000001, precision: 3 },
  displacement: { symbol: 'mm', canonicalToDisplay: 1000, precision: 3 },
  area: { symbol: 'm^2', canonicalToDisplay: 1, precision: 6 },
  secondMomentArea: { symbol: 'm^4', canonicalToDisplay: 1, precision: 9 },
  mass: { symbol: 'kg', canonicalToDisplay: 1, precision: 3 },
  massPerLength: { symbol: 'kg/m', canonicalToDisplay: 1, precision: 3 },
  density: { symbol: 'kg/m^3', canonicalToDisplay: 1, precision: 3 },
};

export function unitProfileFrom(value: unknown): UnitProfile {
  if (!value || typeof value !== 'object') return metricStructuralUnitProfile;
  const candidate = value as Partial<UnitProfile> & { line_load?: UnitProfile['lineLoad']; second_moment_area?: UnitProfile['secondMomentArea']; mass_per_length?: UnitProfile['massPerLength'] };
  const candidateLength = normalizeUnitFormat(metricStructuralUnitProfile.length, candidate.length);
  const isLegacyMetricLength =
    (candidate.id ?? metricStructuralUnitProfile.id) === 'metric_structural' &&
    candidateLength.symbol === 'm' &&
    candidateLength.canonicalToDisplay === 1;
  return {
    ...metricStructuralUnitProfile,
    ...candidate,
    length: isLegacyMetricLength ? metricStructuralUnitProfile.length : candidateLength,
    force: normalizeUnitFormat(metricStructuralUnitProfile.force, candidate.force),
    lineLoad: normalizeUnitFormat(metricStructuralUnitProfile.lineLoad, candidate.lineLoad ?? candidate.line_load),
    moment: normalizeUnitFormat(metricStructuralUnitProfile.moment, candidate.moment),
    stress: normalizeUnitFormat(metricStructuralUnitProfile.stress, candidate.stress),
    displacement: normalizeUnitFormat(metricStructuralUnitProfile.displacement, candidate.displacement),
    area: normalizeUnitFormat(metricStructuralUnitProfile.area!, candidate.area),
    secondMomentArea: normalizeUnitFormat(metricStructuralUnitProfile.secondMomentArea!, candidate.secondMomentArea ?? candidate.second_moment_area),
    mass: normalizeUnitFormat(metricStructuralUnitProfile.mass!, candidate.mass),
    massPerLength: normalizeUnitFormat(metricStructuralUnitProfile.massPerLength!, candidate.massPerLength ?? candidate.mass_per_length),
    density: normalizeUnitFormat(metricStructuralUnitProfile.density!, candidate.density),
  };
}

export function formatQuantity(value: number, kind: QuantityKind, profile = metricStructuralUnitProfile) {
  const unit = unitForKind(kind, profile);
  const displayValue = value * unit.canonicalToDisplay;
  const text = Number.isFinite(displayValue) ? trimFixed(displayValue, unit.precision) : 'n/a';
  return unit.symbol ? `${text} ${unit.symbol}` : text;
}

function unitForKind(kind: QuantityKind, profile: UnitProfile) {
  switch (kind) {
    case 'length':
      return profile.length;
    case 'force':
      return profile.force;
    case 'line_load':
      return profile.lineLoad;
    case 'moment':
      return profile.moment;
    case 'stress':
      return profile.stress;
    case 'displacement':
      return profile.displacement;
    case 'area':
      return profile.area ?? metricStructuralUnitProfile.area!;
    case 'second_moment_area':
      return profile.secondMomentArea ?? profile.second_moment_area ?? metricStructuralUnitProfile.secondMomentArea!;
    case 'mass':
      return profile.mass ?? metricStructuralUnitProfile.mass!;
    case 'mass_per_length':
      return profile.massPerLength ?? profile.mass_per_length ?? metricStructuralUnitProfile.massPerLength!;
    case 'density':
      return profile.density ?? metricStructuralUnitProfile.density!;
  }
}

function normalizeUnitFormat(fallback: UnitProfile['length'], value: UnitProfile['length'] | undefined) {
  return {
    ...fallback,
    ...value,
    canonicalToDisplay: value?.canonicalToDisplay ?? value?.canonical_to_display ?? fallback.canonicalToDisplay,
  };
}

function trimFixed(value: number, precision: number) {
  const text = value.toFixed(precision);
  const trimmed = precision <= 0 ? text.replace(/^-0$/, '0') : text.replace(/\.?0+$/, '').replace(/^-0$/, '0');
  return addThousandsSeparators(trimmed);
}

function addThousandsSeparators(text: string) {
  const [integer, decimal] = text.split('.');
  const sign = integer.startsWith('-') ? '-' : '';
  const unsigned = sign ? integer.slice(1) : integer;
  const grouped = unsigned.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  return `${sign}${grouped}${decimal ? `.${decimal}` : ''}`;
}
