import { Checkbox } from '@/components/ui/checkbox';
import { Field, FieldLabel } from '@/components/ui/field';
import { Separator } from '@/components/ui/separator';
import type { DesignOptionRevisionState, EngineeringScheme, WorkbenchState } from '../../lib/types';
import { SchemeChatPanel } from '../panels/SchemeChatPanel';

export function DesignOptionAgentPanel({
  state,
  scheme,
  revision,
  busy,
  onState,
  onIncludedChange,
}: {
  state: WorkbenchState | null;
  scheme: EngineeringScheme;
  revision: DesignOptionRevisionState | null;
  busy: boolean;
  onState: (state: WorkbenchState) => void;
  onIncludedChange: (included: boolean) => void;
}) {
  const id = `compare-option-${scheme.id.replace(/[^a-z0-9]+/gi, '-')}`;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <Field orientation="horizontal" data-disabled={busy || undefined} className="w-auto shrink-0 justify-end px-3 py-2">
        <Checkbox
          id={id}
          checked={revision?.included ?? false}
          disabled={busy}
          onCheckedChange={(checked) => onIncludedChange(checked === true)}
        />
        <FieldLabel htmlFor={id}>Compare</FieldLabel>
      </Field>
      <Separator />
      <div className="min-h-0 flex-1">
        <SchemeChatPanel
          state={state}
          scheme={scheme}
          surface={`scheme:${scheme.id}`}
          onState={onState}
          showHeader={false}
        />
      </div>
    </div>
  );
}
