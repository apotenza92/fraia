import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { EngineeringScheme } from '../../lib/types';

export function SchemeTabBar({ schemes, activeSchemeId, onSelectScheme }: { schemes: EngineeringScheme[]; activeSchemeId: string | null; onSelectScheme: (id: string) => void }) {
  return (
    <Tabs value={activeSchemeId ?? undefined} onValueChange={onSelectScheme}>
      <TabsList aria-label="Engineering design options" activateOnFocus>
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
    </Tabs>
  );
}
