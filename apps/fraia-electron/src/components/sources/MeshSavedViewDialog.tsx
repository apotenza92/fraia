import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { STLLoader } from 'three/examples/jsm/loaders/STLLoader.js';
import { AlertTriangle, Check, Cuboid, Scissors } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Spinner } from '@/components/ui/spinner';

export type MeshIndexResult = { index: { source_id: string; source_sha256: string; format: 'gltf' | 'glb' | 'obj' | 'stl'; units?: string; coordinate_frame: string; objects: Record<string, { id: string; name?: string; group?: string; bounds: { minimum: number[]; maximum: number[] }; vertex_count: number; triangle_count: number }>; bounds: { minimum: number[]; maximum: number[] }; vertex_count: number; triangle_count: number; diagnostics: Array<{ code: string; object_id?: string; message: string }> }; derivative: unknown; resumed: boolean };

function MeshPreview({ bytes, format, selectedIds, index, onCamera }: { bytes: ArrayBuffer; format: string; selectedIds: Set<string>; index: MeshIndexResult['index']; onCamera: (camera: { position: number[]; target: number[]; up: number[] }) => void }) {
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!host.current) return;
    const scene = new THREE.Scene(); scene.background = new THREE.Color(0xf4f4f5);
    const camera = new THREE.PerspectiveCamera(45, 1, 0.01, 1e7); camera.position.set(4, 3, 4);
    const renderer = new THREE.WebGLRenderer({ antialias: true }); renderer.setPixelRatio(Math.min(devicePixelRatio, 2)); host.current.append(renderer.domElement);
    const controls = new OrbitControls(camera, renderer.domElement); controls.enableDamping = true;
    scene.add(new THREE.HemisphereLight(0xffffff, 0x64748b, 2)); const light = new THREE.DirectionalLight(0xffffff, 2); light.position.set(5, 8, 4); scene.add(light);
    let disposed = false; let model: THREE.Object3D | null = null; const sourceOffset = new THREE.Vector3();
    const finish = (object: THREE.Object3D) => {
      if (disposed) return; model = object;
      object.traverse((item) => { if (item instanceof THREE.Mesh) item.material = new THREE.MeshStandardMaterial({ color: 0x64748b, metalness: .05, roughness: .72, side: THREE.DoubleSide }); }); scene.add(object);
      const sourceBox = new THREE.Box3().setFromObject(object); const size = sourceBox.getSize(new THREE.Vector3()); sourceBox.getCenter(sourceOffset); object.position.sub(sourceOffset); const box = new THREE.Box3().setFromObject(object); const radius = Math.max(size.length() / 2, 1); const distance = radius / Math.tan(THREE.MathUtils.degToRad(camera.fov / 2)) * 1.35; const direction = new THREE.Vector3(1, .72, 1).normalize(); camera.position.copy(direction).multiplyScalar(distance); camera.lookAt(0, 0, 0); controls.target.set(0, 0, 0); camera.near = Math.max(distance / 1000, .001); camera.far = distance * 100; camera.updateProjectionMatrix(); controls.update(); scene.add(new THREE.Box3Helper(box, 0x94a3b8));
    };
    try {
      if (format === 'obj') finish(new OBJLoader().parse(new TextDecoder().decode(bytes)));
      else if (format === 'stl') { const geometry = new STLLoader().parse(bytes); finish(new THREE.Mesh(geometry, new THREE.MeshStandardMaterial({ color: 0x64748b, metalness: .1, roughness: .7 }))); }
      else new GLTFLoader().parse(bytes, '', (gltf) => finish(gltf.scene), () => undefined);
    } catch { /* Index diagnostics remain the authoritative failure. */ }
    renderer.domElement.style.width = '100%'; renderer.domElement.style.height = '100%'; renderer.domElement.style.display = 'block';
    const resize = () => { if (!host.current) return; const { clientWidth: width, clientHeight: height } = host.current; renderer.setSize(width, height, false); camera.aspect = width / Math.max(height, 1); camera.updateProjectionMatrix(); }; const observer = new ResizeObserver(resize); observer.observe(host.current); resize();
    let frame = 0; const animate = () => { controls.update(); renderer.render(scene, camera); frame = requestAnimationFrame(animate); }; animate();
    const capture = () => onCamera({ position: camera.position.clone().add(sourceOffset).toArray(), target: controls.target.clone().add(sourceOffset).toArray(), up: camera.up.toArray() }); controls.addEventListener('change', capture); capture();
    return () => { disposed = true; cancelAnimationFrame(frame); observer.disconnect(); controls.dispose(); renderer.dispose(); renderer.domElement.remove(); model?.traverse((item: any) => { item.geometry?.dispose?.(); item.material?.dispose?.(); }); };
  }, [bytes, format, index]);
  return <div ref={host} role="application" aria-label="Reference mesh preview" className="h-full min-h-72 w-full overflow-hidden rounded-md" data-selected-object-count={selectedIds.size} />;
}

export function MeshSavedViewDialog({ open, projectDir, designId, designName, sourceLabel, indexed, content, onOpenChange, onPrepared }: { open: boolean; projectDir: string; designId: string; designName: string; sourceLabel: string; indexed: MeshIndexResult; content: { sourceSha256: string; bytes: ArrayBuffer }; onOpenChange: (open: boolean) => void; onPrepared: (prepared: any) => Promise<void> }) {
  const objects = Object.values(indexed.index.objects); const [selected, setSelected] = useState<Set<string>>(new Set(objects.map((item) => item.id))); const [label, setLabel] = useState(`${sourceLabel} view`); const [units, setUnits] = useState(indexed.index.units ?? 'mm'); const [unitsToMetres, setUnitsToMetres] = useState(indexed.index.units ? '1' : '0.001'); const [confirmed, setConfirmed] = useState(Boolean(indexed.index.units)); const [sectionEnabled, setSectionEnabled] = useState(false); const [sectionConstant, setSectionConstant] = useState('0'); const [camera, setCamera] = useState({ position: [4,3,4], target: [0,0,0], up: [0,1,0] }); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  async function save() { setBusy(true); setError(null); try { const createdAt = new Date().toISOString(); const prepared = await window.fraia.prepareMeshSavedView({ projectDir, designId, view: { shelf_item_id: `mesh-view-${crypto.randomUUID()}`, label: label.trim(), source_id: indexed.index.source_id, object_ids: [...selected], camera: { ...camera, projection: 'perspective' }, transform: { translation: [0,0,0], rotation_degrees: [0,0,0], scale: [1,1,1] }, orientation: { forward: [0,0,-1], up: [0,1,0] }, scale: Number(unitsToMetres), section_planes: sectionEnabled ? [{ id: 'section-1', normal: [1,0,0], constant: Number(sectionConstant) }] : [], calibration: indexed.index.units ? null : { confirmed, confirmed_by: 'user', confirmed_at: createdAt, units, units_to_metres: Number(unitsToMetres) }, created_at: createdAt, created_by: 'user' } }); await onPrepared(prepared); onOpenChange(false); } catch (caught) { setError(caught instanceof Error ? caught.message : String(caught)); } finally { setBusy(false); } }
  const valid = label.trim() && selected.size && Number(unitsToMetres) > 0 && (indexed.index.units || confirmed);
  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent className="h-[92vh] w-[96vw]! max-w-[96vw]!" data-testid="mesh-saved-view-dialog">
      <DialogHeader>
        <DialogTitle>Save a 3D view</DialogTitle>
        <DialogDescription>Position {sourceLabel}, choose what matters, and save the view for {designName}.</DialogDescription>
      </DialogHeader>
      {error ? <Alert variant="destructive"><AlertTriangle /><AlertTitle>3D reference failed</AlertTitle><AlertDescription>{error}</AlertDescription></Alert> : null}
      <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,2fr)_minmax(18rem,1fr)] gap-4 max-md:grid-cols-1">
        <MeshPreview bytes={content.bytes} format={indexed.index.format} selectedIds={selected} index={indexed.index} onCamera={setCamera} />
        <ScrollArea className="min-h-0">
          <FieldGroup className="pr-3">
            <Alert><Cuboid /><AlertTitle>Read-only reference</AlertTitle><AlertDescription>Fraia keeps this file as evidence. It does not turn the mesh into structure.</AlertDescription></Alert>
            <Field><FieldLabel htmlFor="mesh-view-name">Reference name</FieldLabel><Input id="mesh-view-name" value={label} onChange={(event) => setLabel(event.target.value)} /></Field>
            <Field>
              <FieldLabel>Visible objects</FieldLabel>
              <FieldDescription>Everything is included by default. Clear anything that this view does not need.</FieldDescription>
              {objects.map((item) => <label key={item.id} className="flex items-start gap-2 rounded-md p-2"><Checkbox checked={selected.has(item.id)} aria-label={`Select ${item.name ?? item.id}`} onCheckedChange={(checked) => setSelected((current) => { const next = new Set(current); checked === true ? next.add(item.id) : next.delete(item.id); return next; })} /><span className="min-w-0 truncate font-medium">{item.name ?? item.group ?? 'Mesh object'}</span></label>)}
            </Field>
            {indexed.index.diagnostics.map((diagnostic) => <Alert key={`${diagnostic.code}-${diagnostic.object_id ?? ''}`}><AlertTriangle /><AlertTitle>{diagnostic.code.split('_').join(' ')}</AlertTitle><AlertDescription>{diagnostic.message}</AlertDescription></Alert>)}
            {!indexed.index.units ? <Field><FieldLabel htmlFor="mesh-units">Confirm file scale</FieldLabel><FieldDescription>This file has no units. Tell Fraia how large one file unit is.</FieldDescription><Input id="mesh-units" aria-label="File units" value={units} onChange={(event) => setUnits(event.target.value)} /><Input id="mesh-scale" aria-label="Metres per file unit" inputMode="decimal" value={unitsToMetres} onChange={(event) => setUnitsToMetres(event.target.value)} /><label className="flex items-start gap-2"><Checkbox checked={confirmed} onCheckedChange={(value) => setConfirmed(value === true)} /><span>I checked this scale.</span></label></Field> : <Badge variant="secondary">Units: {indexed.index.units}</Badge>}
            <Collapsible>
              <CollapsibleTrigger render={<Button variant="outline" size="sm" />}>More options</CollapsibleTrigger>
              <CollapsibleContent className="pt-3">
                <Field><FieldLabel>Section plane</FieldLabel><label className="flex items-center gap-2"><Checkbox checked={sectionEnabled} onCheckedChange={(value) => setSectionEnabled(value === true)} /><Scissors />Save an X section plane</label>{sectionEnabled ? <Input aria-label="Section plane offset" inputMode="decimal" value={sectionConstant} onChange={(event) => setSectionConstant(event.target.value)} /> : null}</Field>
              </CollapsibleContent>
            </Collapsible>
            <Collapsible>
              <CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>Reference details</CollapsibleTrigger>
              <CollapsibleContent className="pt-3">
                <div className="text-xs text-muted-foreground"><p>{indexed.index.vertex_count} vertices · {indexed.index.triangle_count} triangles</p><p className="break-all">SHA-256 {content.sourceSha256}</p><p className="break-all">Objects: {[...selected].join(', ')}</p></div>
              </CollapsibleContent>
            </Collapsible>
          </FieldGroup>
        </ScrollArea>
      </div>
      <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button><Button disabled={!valid || busy} onClick={() => void save()}>{busy ? <Spinner data-icon="inline-start" /> : <Check data-icon="inline-start" />}Add design reference</Button></DialogFooter>
    </DialogContent>
  </Dialog>;
}
