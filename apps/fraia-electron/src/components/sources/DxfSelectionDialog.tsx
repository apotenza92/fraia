import { useEffect, useMemo, useState } from 'react';
import { AlertTriangle, Check } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Item, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

type DxfLayer = { name: string; frozen: boolean; hidden: boolean; locked: boolean };
type DxfBlock = { name: string; base_point: [number, number, number]; entity_ids: string[] };
type DxfEntity = {
  id: string;
  entity_type: string;
  layer: string;
  layout: string;
  hidden: boolean;
  frozen: boolean;
  block_name?: string;
};
type DxfDiagnostic = { code: string; message: string; entity_id?: string };

export type DxfIndexResult = {
  index: {
    source_id: string;
    source_sha256: string;
    model_space_name: string;
    paper_layouts: string[];
    units?: string;
    layers: Record<string, DxfLayer>;
    blocks: Record<string, DxfBlock>;
    entities: Record<string, DxfEntity>;
    diagnostics: DxfDiagnostic[];
  };
  derivative: Record<string, unknown>;
  resumed: boolean;
};

export type PreparedDxfSelection = {
  shelf_item: Record<string, unknown> & { id: string; label: string };
  interpretation: Record<string, unknown>;
};

type Props = {
  open: boolean;
  projectDir: string;
  designId: string;
  designName: string;
  source: { label: string };
  indexed: DxfIndexResult;
  interpretationParentRevisionId?: string;
  onOpenChange: (open: boolean) => void;
  onPrepared: (prepared: PreparedDxfSelection) => Promise<void>;
};

const viewRoles = ['plan', 'elevation', 'section', 'detail'] as const;

function numberValue(value: string) {
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

export function DxfSelectionDialog({ open, projectDir, designId, designName, source, indexed, interpretationParentRevisionId, onOpenChange, onPrepared }: Props) {
  const index = indexed.index;
  const layouts = useMemo(() => [index.model_space_name, ...index.paper_layouts], [index.model_space_name, index.paper_layouts]);
  const [layout, setLayout] = useState(layouts[0]);
  const [layers, setLayers] = useState<string[]>([]);
  const [blocks, setBlocks] = useState<string[]>([]);
  const [entityIds, setEntityIds] = useState<string[]>([]);
  const [role, setRole] = useState<(typeof viewRoles)[number]>('plan');
  const [label, setLabel] = useState(`${source.label} selection`);
  const [confirmed, setConfirmed] = useState(false);
  const [origin, setOrigin] = useState({ x: '0', y: '0', z: '0' });
  const [forward, setForward] = useState({ x: '0', y: '1', z: '0' });
  const [up, setUp] = useState({ x: '0', y: '0', z: '1' });
  const [scale, setScale] = useState('1');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const entities = useMemo(() => Object.values(index.entities).filter((entity) => entity.layout === layout), [index.entities, layout]);
  const hasSelection = layers.length > 0 || blocks.length > 0 || entityIds.length > 0;
  const relationNumbers = [origin.x, origin.y, origin.z, forward.x, forward.y, forward.z, up.x, up.y, up.z, scale].map(numberValue);
  const relationValid = relationNumbers.every((value) => value !== null) && Number(scale) > 0;

  useEffect(() => {
    if (!open) return;
    setLayout(layouts[0]);
    setLayers([]);
    setBlocks([]);
    setEntityIds([]);
    setRole('plan');
    setLabel(`${source.label} selection`);
    setConfirmed(false);
    setOrigin({ x: '0', y: '0', z: '0' });
    setForward({ x: '0', y: '1', z: '0' });
    setUp({ x: '0', y: '0', z: '1' });
    setScale('1');
    setError(null);
  }, [layouts, open, source.label]);

  function toggle(current: string[], value: string, checked: boolean) {
    return checked ? [...new Set([...current, value])] : current.filter((item) => item !== value);
  }

  function changeLayout(value: string | null) {
    if (!value) return;
    setLayout(value);
    setLayers([]);
    setBlocks([]);
    setEntityIds([]);
  }

  async function prepare() {
    if (!confirmed || !hasSelection || !relationValid || !label.trim()) return;
    const createdAt = new Date().toISOString();
    setBusy(true);
    setError(null);
    try {
      const prepared = await window.fraia.prepareDxfSelection({
        projectDir,
        designId,
        selection: {
          shelf_item_id: `dxf-selection-${Date.now()}`,
          label: label.trim(),
          source_id: index.source_id,
          layout,
          entity_ids: entityIds,
          layer_names: layers,
          block_names: blocks,
          view_role: role,
          relation_to_design: {
            confirmed: true,
            confirmed_by: 'user',
            confirmed_at: createdAt,
            transform: {
              translation: [Number(origin.x), Number(origin.y), Number(origin.z)],
              rotation_degrees: [0, 0, 0],
              scale: [Number(scale), Number(scale), Number(scale)],
            },
            orientation: {
              forward: [Number(forward.x), Number(forward.y), Number(forward.z)],
              up: [Number(up.x), Number(up.y), Number(up.z)],
            },
            scale: Number(scale),
          },
          created_at: createdAt,
          created_by: 'user',
          interpretation_parent_revision_id: interpretationParentRevisionId,
        },
      });
      await onPrepared(prepared);
      onOpenChange(false);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="h-[90vh] w-[min(94vw,64rem)]! max-w-[min(94vw,64rem)]!" data-testid="dxf-selection-dialog">
        <DialogHeader>
          <DialogTitle>Choose DXF content</DialogTitle>
          <DialogDescription>Select what {designName} needs. Fraia keeps the drawing as evidence and suggests its meaning.</DialogDescription>
        </DialogHeader>
        {error ? <Alert variant="destructive"><AlertTriangle /><AlertTitle>DXF selection failed</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
        <Tabs defaultValue="content" className="min-h-0 flex-1">
          <TabsList className="w-full"><TabsTrigger value="content">Choose content</TabsTrigger><TabsTrigger value="relation">Review</TabsTrigger></TabsList>
          <TabsContent value="content" className="min-h-0">
            <Field>
              <FieldLabel htmlFor="dxf-layout">Layout</FieldLabel>
              <Select items={layouts.map((value) => ({ value, label: value }))} value={layout} onValueChange={changeLayout}>
                <SelectTrigger id="dxf-layout"><SelectValue /></SelectTrigger>
                <SelectContent><SelectGroup>{layouts.map((value) => <SelectItem key={value} value={value}>{value}</SelectItem>)}</SelectGroup></SelectContent>
              </Select>
              <FieldDescription>{entities.length} indexed entities. Drawing units: {index.units ?? 'not declared; scale confirmation is required'}.</FieldDescription>
            </Field>
            <ScrollArea className="mt-3 h-[calc(90vh-19rem)]">
              <div className="flex flex-col gap-3 pr-3">
                <p className="text-sm font-medium">Layers</p>
                {Object.entries(index.layers).map(([name, layer]) => (
                  <Item key={name} variant="outline">
                    <Checkbox aria-label={`Select layer ${name}`} checked={layers.includes(name)} onCheckedChange={(next) => setLayers((current) => toggle(current, name, next === true))} />
                    <ItemContent><ItemTitle>{name}</ItemTitle><ItemDescription>{entities.filter((entity) => entity.layer === name).length} entities in {layout}</ItemDescription></ItemContent>
                    {layer.frozen ? <Badge variant="outline">Frozen</Badge> : null}{layer.hidden ? <Badge variant="outline">Hidden</Badge> : null}{layer.locked ? <Badge variant="outline">Locked</Badge> : null}
                  </Item>
                ))}
                {Object.keys(index.blocks).length > 0 ? <p className="text-sm font-medium">Blocks</p> : null}
                {Object.entries(index.blocks).map(([name, block]) => (
                  <Item key={name} variant="outline">
                    <Checkbox aria-label={`Select block ${name}`} checked={blocks.includes(name)} onCheckedChange={(next) => setBlocks((current) => toggle(current, name, next === true))} />
                    <ItemContent><ItemTitle>{name}</ItemTitle><ItemDescription>{block.entity_ids.length} indexed entities</ItemDescription></ItemContent>
                  </Item>
                ))}
                <p className="text-sm font-medium">Entities in {layout}</p>
                {entities.map((entity) => (
                  <Item key={entity.id} variant="outline">
                    <Checkbox aria-label={`Select entity ${entity.id}`} checked={entityIds.includes(entity.id)} onCheckedChange={(next) => setEntityIds((current) => toggle(current, entity.id, next === true))} />
                    <ItemContent><ItemTitle>{entity.entity_type} {entity.id}</ItemTitle><ItemDescription>Layer {entity.layer}{entity.block_name ? ` · block ${entity.block_name}` : ''}</ItemDescription></ItemContent>
                    {entity.frozen ? <Badge variant="outline">Frozen</Badge> : null}{entity.hidden ? <Badge variant="outline">Hidden</Badge> : null}
                  </Item>
                ))}
                {index.diagnostics.map((diagnostic) => <Alert key={`${diagnostic.code}-${diagnostic.message}`}><AlertTriangle /><AlertTitle>{diagnostic.code.split('_').join(' ')}</AlertTitle><AlertDescription>{diagnostic.message}</AlertDescription></Alert>)}
              </div>
            </ScrollArea>
          </TabsContent>
          <TabsContent value="relation">
            <ScrollArea className="h-[calc(90vh-15rem)]"><FieldGroup className="pr-3">
              <Alert><Check /><AlertTitle>Fraia inferred</AlertTitle><AlertDescription>This starts as a {role} using the selected layout and file units. Correct it below if needed.</AlertDescription></Alert>
              <Field><FieldLabel htmlFor="dxf-reference-name">Name</FieldLabel><Input id="dxf-reference-name" value={label} onChange={(event) => setLabel(event.target.value)} /></Field>
              <Field><FieldLabel htmlFor="dxf-view-role">Drawing view</FieldLabel><Select items={viewRoles.map((value) => ({ value, label: value }))} value={role} onValueChange={(value) => { if (value) setRole(value); }}><SelectTrigger id="dxf-view-role"><SelectValue /></SelectTrigger><SelectContent><SelectGroup>{viewRoles.map((value) => <SelectItem key={value} value={value}>{value}</SelectItem>)}</SelectGroup></SelectContent></Select><FieldDescription>Choose the view shown by this drawing.</FieldDescription></Field>
              {!index.units ? <Field><FieldLabel>Drawing scale</FieldLabel><Input aria-label="DXF-to-design scale" inputMode="decimal" value={scale} onChange={(event) => setScale(event.target.value)} /><FieldDescription>This file does not declare units. Confirm the scale before use.</FieldDescription></Field> : null}
              <Collapsible><CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>Placement details</CollapsibleTrigger><CollapsibleContent><FieldGroup>
                <Field><FieldLabel>Design origin X, Y, Z (m)</FieldLabel><div className="grid grid-cols-3 gap-2">{(['x', 'y', 'z'] as const).map((axis) => <Input key={axis} aria-label={`Origin ${axis.toUpperCase()}`} inputMode="decimal" value={origin[axis]} onChange={(event) => setOrigin((current) => ({ ...current, [axis]: event.target.value }))} />)}</div></Field>
                <Field><FieldLabel>Forward axis X, Y, Z</FieldLabel><div className="grid grid-cols-3 gap-2">{(['x', 'y', 'z'] as const).map((axis) => <Input key={axis} aria-label={`Forward ${axis.toUpperCase()}`} inputMode="decimal" value={forward[axis]} onChange={(event) => setForward((current) => ({ ...current, [axis]: event.target.value }))} />)}</div></Field>
                <Field><FieldLabel>Up axis X, Y, Z</FieldLabel><div className="grid grid-cols-3 gap-2">{(['x', 'y', 'z'] as const).map((axis) => <Input key={axis} aria-label={`Up ${axis.toUpperCase()}`} inputMode="decimal" value={up[axis]} onChange={(event) => setUp((current) => ({ ...current, [axis]: event.target.value }))} />)}</div></Field>
                {index.units ? <Field><FieldLabel>DXF-to-design scale</FieldLabel><Input aria-label="DXF-to-design scale" inputMode="decimal" value={scale} onChange={(event) => setScale(event.target.value)} /></Field> : null}
              </FieldGroup></CollapsibleContent></Collapsible>
              <Field orientation="horizontal"><Checkbox id="confirm-dxf-relation" checked={confirmed} onCheckedChange={(next) => setConfirmed(next === true)} /><FieldLabel htmlFor="confirm-dxf-relation">I checked the drawing view and scale.</FieldLabel></Field>
            </FieldGroup></ScrollArea>
          </TabsContent>
        </Tabs>
        <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button><Button disabled={busy || !confirmed || !hasSelection || !relationValid || !label.trim()} onClick={() => void prepare()}><Check data-icon="inline-start" />Add design reference</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
