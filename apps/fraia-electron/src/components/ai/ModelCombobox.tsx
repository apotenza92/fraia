import { useMemo, useState } from 'react';
import { ChevronsUpDown } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { AgentModelOption, AgentProviderDescriptor } from '@/lib/types';
import { agentModelId, agentModelProviderId } from '@/lib/agentOptions';

function modelName(model: AgentModelOption) {
  return model.displayName ?? model.display_name ?? agentModelId(model);
}

export function ModelCombobox({
  models,
  providers,
  providerId,
  modelId,
  disabled,
  onChange,
}: {
  models: AgentModelOption[];
  providers: AgentProviderDescriptor[];
  providerId: string;
  modelId: string;
  disabled?: boolean;
  onChange: (model: AgentModelOption) => void;
}) {
  const [open, setOpen] = useState(false);
  const selected = models.find((model) => agentModelProviderId(model) === providerId && agentModelId(model) === modelId);
  const grouped = useMemo(() => providers.map((provider) => ({
    provider,
    models: models.filter((model) => agentModelProviderId(model) === provider.id),
  })).filter((group) => group.models.length), [models, providers]);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={(
          <Button type="button" variant="outline" role="combobox" aria-expanded={open} disabled={disabled} className="w-full justify-between font-normal" />
        )}
      >
        <span className="truncate">{selected ? modelName(selected) : modelId || 'Choose a model'}</span>
        <ChevronsUpDown className="opacity-50" />
      </PopoverTrigger>
      <PopoverContent align="start" className="w-[min(24rem,var(--available-width))] gap-0 p-1">
        <Command>
          <CommandInput placeholder="Search models…" aria-label="Search AI models" />
          <CommandList>
            <CommandEmpty>No available models found.</CommandEmpty>
            {grouped.map(({ provider, models: providerModels }) => (
              <CommandGroup key={provider.id} heading={provider.name}>
                {providerModels.map((model) => {
                  const candidateProviderId = agentModelProviderId(model);
                  const candidateModelId = agentModelId(model);
                  return (
                    <CommandItem
                      key={`${candidateProviderId}/${candidateModelId}`}
                      value={`${provider.name} ${modelName(model)} ${candidateModelId}`}
                      data-checked={candidateProviderId === providerId && candidateModelId === modelId}
                      onSelect={() => {
                        onChange(model);
                        setOpen(false);
                      }}
                    >
                      <span className="min-w-0 flex-1 truncate">{modelName(model)}</span>
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
