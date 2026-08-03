import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { EngineeringScheme } from '../../lib/types';

export function SchemeTabBar({ schemes, activeSchemeId, onSelectScheme }: { schemes: EngineeringScheme[]; activeSchemeId: string | null; onSelectScheme: (id: string) => void }) {
  return (
    <Tabs value={activeSchemeId ?? undefined} onValueChange={onSelectScheme} className="min-w-0 gap-0">
      <div className="min-w-0 overflow-x-auto">
      <TabsList aria-label="Engineering design options" variant="line" className="min-w-full justify-start" activateOnFocus>
      {schemes.map((scheme) => {
        return (
          <TabsTrigger
            key={scheme.id}
            aria-controls="fraia-current-model-panel"
            value={scheme.id}
          >
            {scheme.name}
          </TabsTrigger>
        );
      })}
      </TabsList>
      </div>
    </Tabs>
  );
}
