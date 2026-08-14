import { useMemo, useState } from 'react';
import { AlertTriangle, Check } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Item, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { PreparedDxfSelection } from './DxfSelectionDialog';

type IfcTransform = { translation: [number, number, number]; rotation_degrees: [number, number, number]; scale: [number, number, number] };
type IfcObject = { step_id: number; global_id?: string; class_name: string; name?: string; transform: IfcTransform; storey_id?: number };
type IfcStorey = { step_id: number; global_id?: string; name?: string; elevation?: number; transform: IfcTransform };
type IfcGrid = { step_id: number; global_id?: string; name?: string; axis_ids: number[]; transform: IfcTransform };
export type IfcIndexResult = {
  index: {
    source_id: string;
    source_sha256: string;
    file_schema: string[];
    length_unit?: string;
    objects: Record<string, IfcObject>;
    storeys: Record<string, IfcStorey>;
    grids: Record<string, IfcGrid>;
    diagnostics: Array<{ code: string; step_id?: number; message: string }>;
  };
  derivative: Record<string, unknown>;
  resumed: boolean;
};

type Props = {
  open: boolean;
  projectDir: string;
  designId: string;
  designName: string;
  sourceLabel: string;
  indexed: IfcIndexResult;
  interpretationParentRevisionId?: string;
  onOpenChange: (open: boolean) => void;
  onPrepared: (prepared: PreparedDxfSelection) => Promise<void>;
};

function toggle<T>(current: T[], value: T, checked: boolean) {
  return checked ? [...new Set([...current, value])] : current.filter((item) => item !== value);
}

export function IfcSelectionDialog({ open, projectDir, designId, designName, sourceLabel, indexed, interpretationParentRevisionId, onOpenChange, onPrepared }: Props) {
  const index = indexed.index;
  const [label, setLabel] = useState(`${sourceLabel} reference`);
  const [objectIds, setObjectIds] = useState<string[]>([]);
  const [storeyIds, setStoreyIds] = useState<number[]>([]);
  const [gridIds, setGridIds] = useState<number[]>([]);
  const [classNames, setClassNames] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const classes = useMemo(() => [...new Set(Object.values(index.objects).map((object) => object.class_name))].sort(), [index.objects]);
  const hasSelection = objectIds.length + storeyIds.length + gridIds.length + classNames.length > 0;

  async function prepare() {
    if (!hasSelection || !label.trim()) return;
    setBusy(true); setError(null);
    const createdAt = new Date().toISOString();
    try {
      const prepared = await window.fraia.prepareIfcSelection({
        projectDir,
        designId,
        selection: {
          shelf_item_id: `ifc-selection-${Date.now()}`,
          label: label.trim(),
          source_id: index.source_id,
          view_id: `ifc-view-${Date.now()}`,
          object_ids: objectIds,
          storey_ids: storeyIds,
          grid_ids: gridIds,
          class_names: classNames,
          created_at: createdAt,
          created_by: 'user',
          interpretation_parent_revision_id: interpretationParentRevisionId,
        },
      });
      await onPrepared(prepared);
      onOpenChange(false);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally { setBusy(false); }
  }

  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent className="h-[90vh] w-[min(94vw,68rem)]! max-w-[min(94vw,68rem)]!" data-testid="ifc-selection-dialog">
      <DialogHeader><DialogTitle>Choose IFC content</DialogTitle><DialogDescription>Select the parts of {sourceLabel} that {designName} needs. Fraia keeps them as read-only evidence.</DialogDescription></DialogHeader>
      {error ? <Alert variant="destructive"><AlertTriangle /><AlertTitle>IFC selection failed</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
      <Tabs defaultValue="storeys" className="min-h-0 flex-1"><TabsList className="w-full"><TabsTrigger value="storeys">Storeys and grids</TabsTrigger><TabsTrigger value="classes">Classes</TabsTrigger><TabsTrigger value="objects">Objects</TabsTrigger></TabsList>
        <ScrollArea className="h-[calc(90vh-19rem)]"><div className="pr-3">
          <TabsContent value="storeys" className="flex flex-col gap-3">
            {Object.values(index.storeys).map((storey) => <Item key={`storey-${storey.step_id}`} variant="outline"><Checkbox aria-label={`Select storey ${storey.name ?? storey.step_id}`} checked={storeyIds.includes(storey.step_id)} onCheckedChange={(next) => setStoreyIds((current) => toggle(current, storey.step_id, next === true))} /><ItemContent><ItemTitle>{storey.name ?? `Storey ${storey.step_id}`}</ItemTitle><ItemDescription>{storey.elevation === undefined ? 'Elevation not declared' : `Elevation ${storey.elevation}`}</ItemDescription></ItemContent></Item>)}
            {Object.values(index.grids).map((grid) => <Item key={`grid-${grid.step_id}`} variant="outline"><Checkbox aria-label={`Select grid ${grid.name ?? grid.step_id}`} checked={gridIds.includes(grid.step_id)} onCheckedChange={(next) => setGridIds((current) => toggle(current, grid.step_id, next === true))} /><ItemContent><ItemTitle>{grid.name ?? `Grid ${grid.step_id}`}</ItemTitle><ItemDescription>{grid.axis_ids.length} grid ax{grid.axis_ids.length === 1 ? 'is' : 'es'}</ItemDescription></ItemContent></Item>)}
          </TabsContent>
          <TabsContent value="classes" className="flex flex-col gap-3">{classes.map((className) => <Item key={className} variant="outline"><Checkbox aria-label={`Select class ${className}`} checked={classNames.includes(className)} onCheckedChange={(next) => setClassNames((current) => toggle(current, className, next === true))} /><ItemContent><ItemTitle>{className}</ItemTitle><ItemDescription>{Object.values(index.objects).filter((object) => object.class_name === className).length} exact objects</ItemDescription></ItemContent></Item>)}</TabsContent>
          <TabsContent value="objects" className="flex flex-col gap-3">{Object.entries(index.objects).map(([id, object]) => <Item key={id} variant="outline"><Checkbox aria-label={`Select object ${object.name ?? id}`} checked={objectIds.includes(id)} onCheckedChange={(next) => setObjectIds((current) => toggle(current, id, next === true))} /><ItemContent><ItemTitle>{object.name ?? object.class_name}</ItemTitle><ItemDescription>{object.class_name}</ItemDescription></ItemContent><Badge variant="secondary">Fraia inferred</Badge></Item>)}</TabsContent>
          {index.diagnostics.map((diagnostic) => <Alert key={`${diagnostic.code}-${diagnostic.step_id ?? ''}-${diagnostic.message}`} className="mt-3"><AlertTriangle /><AlertTitle>{diagnostic.code.split('_').join(' ')}</AlertTitle><AlertDescription>{diagnostic.message}</AlertDescription></Alert>)}
        </div></ScrollArea>
      </Tabs>
      <Collapsible><CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>Reference details</CollapsibleTrigger><CollapsibleContent><Field><FieldLabel htmlFor="ifc-reference-name">Name</FieldLabel><Input id="ifc-reference-name" value={label} onChange={(event) => setLabel(event.target.value)} /><FieldDescription>{index.file_schema.join(', ')} · length unit {index.length_unit ?? 'not declared'}. Exact IFC IDs and transforms remain attached.</FieldDescription></Field></CollapsibleContent></Collapsible>
      <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button><Button disabled={busy || !hasSelection || !label.trim()} onClick={() => void prepare()}><Check data-icon="inline-start" />Add design reference</Button></DialogFooter>
    </DialogContent>
  </Dialog>;
}
