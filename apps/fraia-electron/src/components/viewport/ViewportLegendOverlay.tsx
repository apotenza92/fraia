import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';

type LegendItem = {
  code: string;
  label: string;
  description?: string;
  tone?: 'node' | 'member' | 'support' | 'load';
};

const elementItems: LegendItem[] = [
  { code: 'N', label: 'Node', tone: 'node' },
  { code: 'M', label: 'Member', tone: 'member' },
  { code: 'S', label: 'Support', tone: 'support' },
  { code: 'L', label: 'Load', tone: 'load' },
];

const groupItems: LegendItem[] = [
  { code: 'GF', label: 'Family group', description: 'Members in this group use the same section family, such as UB, UC, RHS, or SHS.' },
  { code: 'GD', label: 'Designation group', description: 'Members in this group use the same catalogue designation within a family.' },
  { code: 'GS', label: 'Support group', description: 'Supports in this group use the same restraint type.', tone: 'support' },
];

function LegendSection({ title, items }: { title: string; items: LegendItem[] }) {
  return (
    <section className="flex flex-col gap-2">
      <h3 className="font-semibold text-muted-foreground">{title}</h3>
      <div className="flex flex-col gap-3">
        {items.map((item) => (
          <div key={item.code} className="flex flex-col gap-0.5">
            <div className="flex flex-nowrap items-baseline gap-2">
              <span className="w-8 font-mono font-semibold">{item.code}</span>
              <span className="font-semibold">{item.label}</span>
            </div>
            {item.description && <p className="ml-10 text-xs text-muted-foreground">{item.description}</p>}
          </div>
        ))}
      </div>
    </section>
  );
}

export function ViewportLegendContent() {
  return (
    <div className="grid gap-4 p-3 sm:grid-cols-[1fr_2fr]">
        <LegendSection title="Elements" items={elementItems} />
        <LegendSection title="Groups" items={groupItems} />
    </div>
  );
}

export function LegendDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  return (
    <Dialog open={open} onOpenChange={(next) => { if (!next) onClose(); }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Legend</DialogTitle>
        </DialogHeader>
        <ViewportLegendContent />
      </DialogContent>
    </Dialog>
  );
}
