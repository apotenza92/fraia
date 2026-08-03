import { useEffect, useMemo, useState } from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Empty, EmptyDescription } from '@/components/ui/empty';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { WorkbenchState } from '../../lib/types';
import { projectDirOf } from '../../lib/defaultProject';
import { buildSchemeWorkspace } from '../../lib/scene';
import { displayMembersFor, memberLabel } from '../../lib/renderMembers';

type TabKey = 'input' | 'dat' | 'nodes' | 'reactions' | 'stresses' | 'execution';

type RawAnalysis = {
  runId?: string;
  runDir?: string;
  solverResults?: RawSolverResult[];
  diagnostics?: unknown[];
  summaryMd?: string;
  comparison?: {
    optionResults?: Array<{
      optionId?: string;
      optionLabel?: string;
      candidateResults?: Array<{ optionId?: string; optionLabel?: string }>;
    }>;
  };
};

type RawSolverResult = {
  runId?: string;
  optionId?: string;
  coordinationGroupId?: string;
  sectionId?: string;
  solver?: string;
  maxMomentKnm?: number | null;
  max_moment_knm?: number | null;
  maxShearKn?: number | null;
  max_shear_kn?: number | null;
  maxDeflectionMm?: number | null;
  max_deflection_mm?: number | null;
  maxStressMpa?: number | null;
  max_stress_mpa?: number | null;
  maxReactionKn?: number | null;
  max_reaction_kn?: number | null;
  maxUtilization?: number | null;
  max_utilization?: number | null;
  candidateInput?: { memberIds?: string[]; member_ids?: string[]; standardisationPolicy?: string; standardisation_policy?: string };
  compiledInputs?: Array<{ jobName?: string; job_name?: string; comboId?: string; combo_id?: string; inputDeck?: string; input_deck?: string; nodeCount?: number; node_count?: number; elementCount?: number; element_count?: number }>;
  executions?: Array<{ jobName?: string; job_name?: string; command?: string[]; workingDir?: string; working_dir?: string; outcome?: string; exitCode?: number | null; exit_code?: number | null; stdout?: string; stderr?: string }>;
  nodeDisplacements?: Array<{ nodeId?: string; node_id?: string; xM?: number; x_m?: number; yM?: number; y_m?: number; uxM?: number; ux_m?: number; uyM?: number; uy_m?: number }>;
  supportReactions?: Array<{ nodeId?: string; node_id?: string; xM?: number; x_m?: number; yM?: number; y_m?: number; fxN?: number; fx_n?: number; fyN?: number; fy_n?: number }>;
  elementStresses?: Array<{ elementId?: string; element_id?: string; role?: string; pointCount?: number; point_count?: number; maxAbsSxxPa?: number; max_abs_sxx_pa?: number; maxAbsSxyPa?: number; max_abs_sxy_pa?: number }>;
  rawFiles?: Array<{ jobName?: string; job_name?: string; workingDir?: string; working_dir?: string; files?: { inp?: string; dat?: string; sta?: string; cvg?: string } }>;
  realizationDiagnostics?: unknown[];
};

type LabelMaps = {
  optionLabels: Map<string, string>;
  memberLabels: Map<string, string>;
  groupLabels: Map<string, string>;
};

function latestRunId(state: WorkbenchState | null) {
  const run = state?.latestRunSummary ?? state?.latest_run_summary;
  const designOption = state?.latestDesignOptionAnalysis ?? state?.latest_design_option_analysis;
  return designOption?.runId ?? designOption?.run_id ?? run?.runId ?? run?.run_id ?? undefined;
}

function value<T>(camel: T | undefined, snake: T | undefined): T | undefined {
  return camel ?? snake;
}

function sectionFamily(sectionId: string) {
  const upper = sectionId.toUpperCase();
  return ['PFC', 'RHS', 'SHS', 'CHS', 'UB', 'UC', 'EA'].find((family) => upper.endsWith(family)) ?? 'unknown';
}

function sectionSizeSortValue(sectionId: string) {
  const match = sectionId.match(/\d+(?:\.\d+)?/);
  return match ? Number(match[0]) : Number.POSITIVE_INFINITY;
}

function mapKey(optionId: string | undefined, id: string | undefined) {
  return `${optionId ?? ''}::${id ?? ''}`;
}

function titleRole(role: string | undefined) {
  const raw = role?.trim() || 'member';
  return raw
    .split(/[_\s-]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function fallbackMemberLabel(memberId: string) {
  const localId = memberId.split('::').pop() ?? memberId;
  return `Member ${localId.toUpperCase()}`;
}

function buildLabelMaps(state: WorkbenchState | null, raw: RawAnalysis | null): LabelMaps {
  const optionLabels = new Map<string, string>();
  const memberLabels = new Map<string, string>();
  const groupLabels = new Map<string, string>();

  for (const option of raw?.comparison?.optionResults ?? []) {
    if (option.optionId && option.optionLabel) optionLabels.set(option.optionId, option.optionLabel);
    for (const candidate of option.candidateResults ?? []) {
      if (candidate.optionId && candidate.optionLabel) optionLabels.set(candidate.optionId, candidate.optionLabel);
    }
  }

  const backendSchemes = [...(state?.designSchemes ?? []), ...(state?.design_schemes ?? [])];
  for (const scheme of backendSchemes) {
    const id = scheme?.id;
    const label = scheme?.label ?? scheme?.intent?.label;
    if (id && label) optionLabels.set(id, label);
  }

  for (const scheme of buildSchemeWorkspace(state).schemes) {
    optionLabels.set(scheme.id, scheme.name);
    for (const displayMember of displayMembersFor(scheme.scene)) {
      const label = memberLabel(displayMember);
      for (const segment of displayMember.segments) {
        memberLabels.set(mapKey(scheme.id, segment.memberId), label);
        if (!memberLabels.has(mapKey(undefined, segment.memberId))) {
          memberLabels.set(mapKey(undefined, segment.memberId), label);
        }
      }
    }
    for (const member of scheme.scene.members) {
      const label = [
        member.coordinationGroupLabel,
        member.familyGroupLabel,
        member.sectionCoordination?.groupLabel,
      ].find((item) => item && item.trim());
      if (member.coordinationGroupId && label) {
        groupLabels.set(mapKey(scheme.id, member.coordinationGroupId), label);
        if (!groupLabels.has(mapKey(undefined, member.coordinationGroupId))) {
          groupLabels.set(mapKey(undefined, member.coordinationGroupId), label);
        }
      }
      if (!memberLabels.has(mapKey(scheme.id, member.id))) {
        memberLabels.set(mapKey(scheme.id, member.id), `${titleRole(member.role)} ${member.id}`);
      }
    }
  }

  for (const member of state?.scene?.members ?? []) {
    if (!memberLabels.has(mapKey(undefined, member.id))) {
      memberLabels.set(mapKey(undefined, member.id), `${titleRole(member.role)} ${member.id}`);
    }
    const groupId = value(member.coordinationGroupId, member.coordination_group_id);
    const groupLabel = value(member.coordinationGroupLabel, member.coordination_group_label)
      ?? value(member.familyGroupLabel, member.family_group_label)
      ?? value(member.sectionCoordination?.groupLabel, member.section_coordination?.group_label);
    if (groupId && groupLabel && !groupLabels.has(mapKey(undefined, groupId))) {
      groupLabels.set(mapKey(undefined, groupId), groupLabel);
    }
  }

  return { optionLabels, memberLabels, groupLabels };
}

function optionLabel(result: RawSolverResult, labels?: LabelMaps) {
  return labels?.optionLabels.get(result.optionId ?? '') ?? result.optionId ?? 'option';
}

function groupLabel(result: RawSolverResult, labels?: LabelMaps) {
  return labels?.groupLabels.get(mapKey(result.optionId, result.coordinationGroupId))
    ?? labels?.groupLabels.get(mapKey(undefined, result.coordinationGroupId))
    ?? result.coordinationGroupId
    ?? '';
}

function candidateKey(result: RawSolverResult) {
  return [result.optionId, result.coordinationGroupId, result.sectionId].filter(Boolean).join(' / ');
}

function firstExecution(result: RawSolverResult) {
  return result.executions?.[0] ?? null;
}

function firstInputDeck(result: RawSolverResult) {
  return result.rawFiles?.[0]?.files?.inp || result.compiledInputs?.[0]?.inputDeck || result.compiledInputs?.[0]?.input_deck || '';
}

function firstDat(result: RawSolverResult) {
  return result.rawFiles?.[0]?.files?.dat || '';
}

function formatNumber(value: unknown, digits = 6) {
  const number = Number(value);
  return Number.isFinite(number) ? number.toFixed(digits) : '';
}

function metricValue(result: RawSolverResult, camel: keyof RawSolverResult, snake: keyof RawSolverResult) {
  const direct = value(result[camel] as number | null | undefined, result[snake] as number | null | undefined);
  return typeof direct === 'number' && Number.isFinite(direct) ? direct : null;
}

function maxExtractedDisplacementMm(result: RawSolverResult) {
  const maxM = (result.nodeDisplacements ?? []).reduce((current, node) => {
    const ux = Number(value(node.uxM, node.ux_m) ?? 0);
    const uy = Number(value(node.uyM, node.uy_m) ?? 0);
    const resultant = Math.hypot(ux, uy);
    return Number.isFinite(resultant) ? Math.max(current, resultant) : current;
  }, 0);
  return maxM > 0 ? maxM * 1000 : null;
}

function maxExtractedStressMpa(result: RawSolverResult) {
  const maxPa = (result.elementStresses ?? []).reduce((current, stress) => {
    const sxx = Number(value(stress.maxAbsSxxPa, stress.max_abs_sxx_pa) ?? 0);
    const sxy = Number(value(stress.maxAbsSxyPa, stress.max_abs_sxy_pa) ?? 0);
    const maxStress = Math.max(Math.abs(sxx), Math.abs(sxy));
    return Number.isFinite(maxStress) ? Math.max(current, maxStress) : current;
  }, 0);
  return maxPa > 0 ? maxPa / 1_000_000 : null;
}

function maxExtractedReactionKn(result: RawSolverResult) {
  const maxN = (result.supportReactions ?? []).reduce((current, reaction) => {
    const fx = Number(value(reaction.fxN, reaction.fx_n) ?? 0);
    const fy = Number(value(reaction.fyN, reaction.fy_n) ?? 0);
    const resultant = Math.hypot(fx, fy);
    return Number.isFinite(resultant) ? Math.max(current, resultant) : current;
  }, 0);
  return maxN > 0 ? maxN / 1000 : null;
}

function resultMetrics(result: RawSolverResult) {
  return {
    momentKnm: metricValue(result, 'maxMomentKnm', 'max_moment_knm'),
    shearKn: metricValue(result, 'maxShearKn', 'max_shear_kn'),
    deflectionMm: metricValue(result, 'maxDeflectionMm', 'max_deflection_mm') ?? maxExtractedDisplacementMm(result),
    stressMpa: metricValue(result, 'maxStressMpa', 'max_stress_mpa') ?? maxExtractedStressMpa(result),
    reactionKn: metricValue(result, 'maxReactionKn', 'max_reaction_kn') ?? maxExtractedReactionKn(result),
    utilization: metricValue(result, 'maxUtilization', 'max_utilization'),
  };
}

function compareText(left: string, right: string) {
  return left.localeCompare(right, undefined, { numeric: true, sensitivity: 'base' });
}

function resultSortValue(result: RawSolverResult) {
  const metrics = resultMetrics(result);
  return metrics.utilization ?? metrics.deflectionMm ?? metrics.stressMpa ?? metrics.momentKnm ?? metrics.shearKn ?? metrics.reactionKn ?? -1;
}

function sortRawResults(results: RawSolverResult[], labels: LabelMaps) {
  return [...results].sort((left, right) => {
    const leftMembers = memberNames(left, labels).join(', ');
    const rightMembers = memberNames(right, labels).join(', ');
    return compareText(optionLabel(left, labels), optionLabel(right, labels))
      || compareText(leftMembers, rightMembers)
      || compareText(sectionFamily(left.sectionId ?? ''), sectionFamily(right.sectionId ?? ''))
      || sectionSizeSortValue(left.sectionId ?? '') - sectionSizeSortValue(right.sectionId ?? '')
      || resultSortValue(left) - resultSortValue(right)
      || compareText(left.sectionId ?? '', right.sectionId ?? '');
  });
}

function statusText(result: RawSolverResult) {
  const execution = firstExecution(result);
  return execution?.outcome ?? 'not run';
}

function exitCode(result: RawSolverResult) {
  const execution = firstExecution(result);
  return value(execution?.exitCode, execution?.exit_code);
}

function memberIds(result: RawSolverResult) {
  return value(result.candidateInput?.memberIds, result.candidateInput?.member_ids) ?? [];
}

function memberNames(result: RawSolverResult, labels?: LabelMaps) {
  return memberIds(result).map((id) => (
    labels?.memberLabels.get(mapKey(result.optionId, id))
    ?? labels?.memberLabels.get(mapKey(undefined, id))
    ?? fallbackMemberLabel(id)
  ));
}

function RawTextPanel({ text, empty }: { text: string; empty: string }) {
  if (!text) {
    return (
      <Empty className="h-[420px]">
        <EmptyDescription>{empty}</EmptyDescription>
      </Empty>
    );
  }
  return (
    <Card className="h-[420px] overflow-auto p-3">
      <pre className="whitespace-pre-wrap font-mono text-sm">{text || empty}</pre>
    </Card>
  );
}

function DenseTable({ columns, rows }: { columns: string[]; rows: Array<Array<string | number | null | undefined>> }) {
  return (
    <div className="overflow-auto">
      <Table className="min-w-[720px]">
        <TableHeader>
          <TableRow>
            {columns.map((column) => (
              <TableHead key={column}>{column}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.length ? rows.map((row, index) => (
            <TableRow key={index}>
              {row.map((cell, cellIndex) => (
                <TableCell key={cellIndex}>
                  <code className="font-mono text-sm">{cell ?? ''}</code>
                </TableCell>
              ))}
            </TableRow>
          )) : (
            <TableRow>
              <TableCell colSpan={columns.length}>
                <Empty className="min-h-28">
                  <EmptyDescription>No extracted rows.</EmptyDescription>
                </Empty>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

function CandidateTable({ results, selected, labels, onSelect }: { results: RawSolverResult[]; selected: RawSolverResult | null; labels: LabelMaps; onSelect: (result: RawSolverResult) => void }) {
  return (
    <div className="overflow-auto">
      <Table className="min-w-[1320px]">
        <TableHeader>
          <TableRow>
            <TableHead>Design option</TableHead>
            <TableHead>Members</TableHead>
            <TableHead>Group</TableHead>
            <TableHead>Shape</TableHead>
            <TableHead>Size</TableHead>
            <TableHead>Section</TableHead>
            <TableHead><div className="text-right">Moment kNm</div></TableHead>
            <TableHead><div className="text-right">Shear kN</div></TableHead>
            <TableHead><div className="text-right">Defl mm</div></TableHead>
            <TableHead><div className="text-right">Stress MPa</div></TableHead>
            <TableHead><div className="text-right">Reaction kN</div></TableHead>
            <TableHead>Outcome</TableHead>
            <TableHead>Exit</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {results.length ? results.map((result, index) => {
            const active = selected === result;
            const metrics = resultMetrics(result);
            const shape = sectionFamily(result.sectionId ?? '');
            const size = sectionSizeSortValue(result.sectionId ?? '');
            const members = memberNames(result, labels);
            const option = optionLabel(result, labels);
            const group = groupLabel(result, labels);
            return (
              <TableRow
                key={`${candidateKey(result)}-${index}`}
                data-state={active ? 'selected' : undefined}
              >
                <TableCell title={`${option} (${result.optionId ?? ''})`}>
                  <Button
                    type="button"
                    variant="link"
                    aria-pressed={active}
                    onClick={() => onSelect(result)}
                    className="h-auto max-w-[260px] justify-start p-0"
                  >
                    <span className="truncate">{option}</span>
                  </Button>
                </TableCell>
                <TableCell title={`${members.join(', ')} (${memberIds(result).join(', ')})`}><div className="max-w-[220px] truncate text-muted-foreground">{members.join(', ')}</div></TableCell>
                <TableCell title={`${group} (${result.coordinationGroupId ?? ''})`}><div className="max-w-[220px] truncate">{group}</div></TableCell>
                <TableCell><code className="font-mono text-sm">{shape}</code></TableCell>
                <TableCell><div className="text-right"><code className="font-mono text-sm">{Number.isFinite(size) ? size : ''}</code></div></TableCell>
                <TableCell><code className="font-mono text-sm">{result.sectionId}</code></TableCell>
                <TableCell><div className="text-right"><code className="font-mono text-sm">{formatNumber(metrics.momentKnm, 2)}</code></div></TableCell>
                <TableCell><div className="text-right"><code className="font-mono text-sm">{formatNumber(metrics.shearKn, 2)}</code></div></TableCell>
                <TableCell><div className="text-right"><code className="font-mono text-sm">{formatNumber(metrics.deflectionMm, 3)}</code></div></TableCell>
                <TableCell><div className="text-right"><code className="font-mono text-sm">{formatNumber(metrics.stressMpa, 2)}</code></div></TableCell>
                <TableCell><div className="text-right"><code className="font-mono text-sm">{formatNumber(metrics.reactionKn, 2)}</code></div></TableCell>
                <TableCell><code className="font-mono text-sm">{statusText(result)}</code></TableCell>
                <TableCell><code className="font-mono text-sm">{exitCode(result) ?? ''}</code></TableCell>
              </TableRow>
            );
          }) : (
            <TableRow>
              <TableCell colSpan={13}>
                <Empty className="min-h-28">
                  <EmptyDescription>No raw CalculiX candidates found.</EmptyDescription>
                </Empty>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

function DetailPanel({ result }: { result: RawSolverResult | null }) {
  const [tab, setTab] = useState<TabKey>('input');
  useEffect(() => setTab('input'), [result]);

  if (!result) {
    return <Card><CardContent><p className="text-muted-foreground">Run design-option analysis, then select a candidate.</p></CardContent></Card>;
  }

  const execution = firstExecution(result);
  const tabs: Array<[TabKey, string]> = [['input', 'Input deck'], ['dat', 'Raw .dat'], ['nodes', 'Nodes'], ['reactions', 'Reactions'], ['stresses', 'Element stresses'], ['execution', 'Execution']];
  const executionText = JSON.stringify({
    command: execution?.command,
    workingDir: value(execution?.workingDir, execution?.working_dir),
    outcome: execution?.outcome,
    exitCode: value(execution?.exitCode, execution?.exit_code),
    stdout: execution?.stdout,
    stderr: execution?.stderr,
    rawFiles: result.rawFiles,
    realizationDiagnostics: result.realizationDiagnostics,
  }, null, 2);

  return (
    <Card>
      <CardHeader>
        <p className="text-sm text-muted-foreground">Active CalculiX candidate</p>
        <pre className="rounded-md bg-muted p-2 font-mono text-sm">{candidateKey(result)}</pre>
        <p className="text-xs text-muted-foreground">{value(execution?.workingDir, execution?.working_dir) ?? 'No working directory'}</p>
      </CardHeader>
      <Tabs value={tab} onValueChange={(value) => setTab(value as TabKey)}>
        <TabsList activateOnFocus>
          {tabs.map(([key, label]) => (
            <TabsTrigger key={key} value={key}>{label}</TabsTrigger>
          ))}
        </TabsList>
        <CardContent className="flex flex-col gap-3">
          <TabsContent value="input">
            <RawTextPanel text={firstInputDeck(result)} empty="No .inp deck was captured." />
          </TabsContent>
          <TabsContent value="dat">
            <RawTextPanel text={firstDat(result)} empty="No .dat output was captured." />
          </TabsContent>
          <TabsContent value="nodes">
            <DenseTable
              columns={['node id', 'x m', 'y m', 'ux m', 'uy m']}
              rows={(result.nodeDisplacements ?? []).map((node) => [
                value(node.nodeId, node.node_id),
                formatNumber(value(node.xM, node.x_m)),
                formatNumber(value(node.yM, node.y_m)),
                formatNumber(value(node.uxM, node.ux_m), 9),
                formatNumber(value(node.uyM, node.uy_m), 9),
              ])}
            />
          </TabsContent>
          <TabsContent value="reactions">
            <DenseTable
              columns={['node id', 'x m', 'y m', 'fx N', 'fy N']}
              rows={(result.supportReactions ?? []).map((reaction) => [
                value(reaction.nodeId, reaction.node_id),
                formatNumber(value(reaction.xM, reaction.x_m)),
                formatNumber(value(reaction.yM, reaction.y_m)),
                formatNumber(value(reaction.fxN, reaction.fx_n), 3),
                formatNumber(value(reaction.fyN, reaction.fy_n), 3),
              ])}
            />
          </TabsContent>
          <TabsContent value="stresses">
            <DenseTable
              columns={['member id', 'role', 'points', 'max |sxx| Pa', 'max |sxy| Pa']}
              rows={(result.elementStresses ?? []).map((stress) => [
                value(stress.elementId, stress.element_id),
                stress.role,
                value(stress.pointCount, stress.point_count),
                formatNumber(value(stress.maxAbsSxxPa, stress.max_abs_sxx_pa), 3),
                formatNumber(value(stress.maxAbsSxyPa, stress.max_abs_sxy_pa), 3),
              ])}
            />
          </TabsContent>
          <TabsContent value="execution">
            <RawTextPanel text={executionText} empty="No execution data was captured." />
          </TabsContent>
        </CardContent>
      </Tabs>
    </Card>
  );
}

export function ResultsWorkspace({ state, requestedRunId }: { state: WorkbenchState | null; requestedRunId?: string | null }) {
  const [raw, setRaw] = useState<RawAnalysis | null>(null);
  const [selected, setSelected] = useState<RawSolverResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const projectDir = projectDirOf(state);
  const runId = requestedRunId ?? latestRunId(state);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    window.fraia.rawDesignOptionAnalysis({ projectDir, runId })
      .then((response: RawAnalysis) => {
        if (cancelled) return;
        const results = response?.solverResults ?? [];
        setRaw(response);
        setSelected((current) => current && results.includes(current) ? current : results[0] ?? null);
      })
      .catch((caught: Error) => {
        if (cancelled) return;
        setRaw(null);
        setSelected(null);
        setError(caught.message || 'Could not load raw design-option analysis.');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [projectDir, runId]);

  const labels = useMemo(() => buildLabelMaps(state, raw), [state, raw]);
  const results = useMemo(() => sortRawResults(raw?.solverResults ?? [], labels), [raw, labels]);

  return (
    <div className="h-full flex-1 overflow-auto">
      <div className="flex flex-col gap-3 p-3">
        <Card>
          <CardContent className="flex items-end justify-between gap-4">
            <div>
              <h1 className="text-2xl font-semibold">Raw CalculiX results</h1>
              <p className="text-xs text-muted-foreground">Verification view for solver inputs and extracted output. No design winner is chosen here.</p>
            </div>
            <div className="flex flex-col items-end gap-1">
              <code className="font-mono text-sm">{loading ? 'Loading raw run...' : raw?.runId ?? runId ?? 'No design-option run'}</code>
              <p className="max-w-md truncate text-xs text-muted-foreground" title={raw?.runDir}>{raw?.runDir}</p>
            </div>
          </CardContent>
        </Card>

        {error ? (
          <Alert><AlertDescription>{error}</AlertDescription></Alert>
        ) : (
          <>
            <CandidateTable results={results} selected={selected} labels={labels} onSelect={setSelected} />
            <DetailPanel result={selected} />
          </>
        )}
      </div>
    </div>
  );
}
