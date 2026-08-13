import { CircleHelp } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Field, FieldLabel } from '@/components/ui/field';
import { Kbd, KbdGroup } from '@/components/ui/kbd';
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from '@/components/ui/popover';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import {
  VIEWPORT_NAVIGATION_PROFILES,
  handedMouseButton,
  handedViewportNavigationLabel,
  isViewportMouseHandedness,
  isViewportNavigationAction,
  isViewportNavigationProfileId,
  viewportNavigationProfile,
  type ViewportCustomNavigationSettings,
  type ViewportNavigationAction,
  type ViewportNavigationProfileId,
  type ViewportMouseHandedness,
} from '@/lib/viewportNavigation';
import {
  DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
  viewportSelectionDescription,
  type ViewportCustomSelectionSettings,
} from '@/lib/viewportSelection';

export type ViewportHelpShortcut = {
  id: string;
  keys: string[];
  label: string;
};

const CUSTOM_ACTION_ITEMS = [
  { value: 'rotate', label: 'Rotate' },
  { value: 'pan', label: 'Pan' },
  { value: 'zoom', label: 'Zoom' },
  { value: 'none', label: 'No camera action' },
] satisfies { value: ViewportNavigationAction; label: string }[];

function CustomButtonField({
  button,
  handedness,
  value,
  onValue,
}: {
  button: 'left' | 'middle' | 'right';
  handedness: ViewportMouseHandedness;
  value: ViewportNavigationAction;
  onValue: (value: ViewportNavigationAction) => void;
}) {
  const physicalButton = handedMouseButton(button, handedness);
  const label = `${physicalButton[0].toUpperCase()}${physicalButton.slice(1)} button`;
  return (
    <Field orientation="horizontal">
      <FieldLabel htmlFor={`viewport-custom-${button}`} className="min-w-24">{label}</FieldLabel>
      <Select
        items={CUSTOM_ACTION_ITEMS}
        value={value}
        onValueChange={(nextValue) => {
          if (isViewportNavigationAction(nextValue)) onValue(nextValue);
        }}
      >
        <SelectTrigger id={`viewport-custom-${button}`} className="min-w-0 flex-1" aria-label={`${label} action`}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent alignItemWithTrigger={false}>
          <SelectGroup>
            {CUSTOM_ACTION_ITEMS.map((item) => (
              <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </Field>
  );
}

const CUSTOM_SELECTION_FIELDS = [
  { key: 'pickBehavior', label: 'Direct click', items: [['replace', 'Replace'], ['add', 'Add'], ['toggle', 'Toggle']] },
  { key: 'modifierStyle', label: 'Modifiers', items: [['shift-add-primary-remove', 'Shift add · Ctrl/Cmd remove'], ['shift-remove-primary-toggle', 'Shift remove · Ctrl/Cmd toggle'], ['primary-toggle', 'Ctrl/Cmd toggle'], ['none', 'None']] },
  { key: 'emptyBehavior', label: 'Blank click', items: [['clear', 'Clear selection'], ['start-window', 'Start window'], ['ignore', 'Do nothing']] },
  { key: 'windowGesture', label: 'Window', items: [['drag', 'Drag'], ['two-click', 'Two clicks']] },
  { key: 'windowBehavior', label: 'Window result', items: [['replace', 'Replace'], ['add', 'Add'], ['toggle', 'Toggle']] },
  { key: 'forceWindowModifier', label: 'Force window', items: [['alt', 'Alt'], ['shift', 'Shift'], ['none', 'None']] },
] as const;

function CustomSelectionFields({
  settings,
  onSettings,
}: {
  settings: ViewportCustomSelectionSettings;
  onSettings: (settings: ViewportCustomSelectionSettings) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <p className="font-medium">Custom selection</p>
      {CUSTOM_SELECTION_FIELDS.map((field) => (
        <Field key={field.key} orientation="horizontal">
          <FieldLabel htmlFor={`viewport-selection-${field.key}`} className="min-w-24">{field.label}</FieldLabel>
          <Select
            items={field.items.map(([value, label]) => ({ value, label }))}
            value={settings[field.key]}
            onValueChange={(value) => {
              if (field.items.some(([candidate]) => candidate === value)) {
                onSettings({ ...settings, [field.key]: value });
              }
            }}
          >
            <SelectTrigger id={`viewport-selection-${field.key}`} className="min-w-0 flex-1" aria-label={field.label}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent alignItemWithTrigger={false}>
              <SelectGroup>
                {field.items.map(([value, label]) => <SelectItem key={value} value={value}>{label}</SelectItem>)}
              </SelectGroup>
            </SelectContent>
          </Select>
        </Field>
      ))}
    </div>
  );
}

function Mapping({ label, value }: { label: string; value: string }) {
  return (
    <Tooltip>
      <TooltipTrigger render={<span className="flex min-w-0 items-center" />}>
        <span className="truncate"><span className="font-medium text-foreground">{label}</span> · {value}</span>
      </TooltipTrigger>
      <TooltipContent>{label}: {value}</TooltipContent>
    </Tooltip>
  );
}

function ControlRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[4rem_1fr] items-center gap-2">
      <span className="font-medium">{label}</span>
      <span className="text-muted-foreground">{value}</span>
    </div>
  );
}

function ShortcutRow({ shortcut }: { shortcut: ViewportHelpShortcut }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span>{shortcut.label}</span>
      <KbdGroup>
        {shortcut.keys.map((key) => <Kbd key={key}>{key}</Kbd>)}
      </KbdGroup>
    </div>
  );
}

export function ViewportHelpBar({
  availableWidth,
  status,
  navigationProfileId,
  customNavigationSettings,
  customSelectionSettings = DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS,
  mouseHandedness,
  contextualShortcuts,
  onNavigationProfileId,
  onCustomNavigationSettings,
  onCustomSelectionSettings = () => {},
  onMouseHandedness,
}: {
  availableWidth: number;
  status: string;
  navigationProfileId: ViewportNavigationProfileId;
  customNavigationSettings: ViewportCustomNavigationSettings;
  customSelectionSettings?: ViewportCustomSelectionSettings;
  mouseHandedness: ViewportMouseHandedness;
  contextualShortcuts: ViewportHelpShortcut[];
  onNavigationProfileId: (profileId: ViewportNavigationProfileId) => void;
  onCustomNavigationSettings: (settings: ViewportCustomNavigationSettings) => void;
  onCustomSelectionSettings?: (settings: ViewportCustomSelectionSettings) => void;
  onMouseHandedness: (handedness: ViewportMouseHandedness) => void;
}) {
  const profile = viewportNavigationProfile(navigationProfileId, customNavigationSettings);
  const rotateLabel = handedViewportNavigationLabel(profile.essentials.rotate, mouseHandedness);
  const panLabel = handedViewportNavigationLabel(profile.essentials.pan, mouseHandedness);
  const zoomLabel = handedViewportNavigationLabel(profile.essentials.zoom, mouseHandedness);
  const profileItems = VIEWPORT_NAVIGATION_PROFILES.map((item) => ({ value: item.id, label: item.label }));
  const selection = viewportSelectionDescription(navigationProfileId, customSelectionSettings);
  const showCameraMappings = availableWidth >= 520;

  return (
    <div data-testid="viewport-help-bar" className="h-10 shrink-0 bg-background">
      <Separator />
      <div className="flex h-[39px] min-w-0 items-center gap-4 px-3">
        <div data-testid="viewport-help-status" className="min-w-0 flex-1 truncate text-sm text-muted-foreground">
          {status} · {selection.short}
        </div>

        {showCameraMappings ? (
          <div data-testid="viewport-help-essentials" className="flex min-w-0 items-center gap-4 text-sm text-muted-foreground">
            <Mapping label="Rotate" value={rotateLabel} />
            <Mapping label="Pan" value={panLabel} />
            <Mapping label="Zoom" value={zoomLabel} />
          </div>
        ) : null}

        <Popover>
          <PopoverTrigger render={<Button aria-label="Controls" variant="ghost" size="icon" className="shrink-0" />}>
            <CircleHelp aria-hidden="true" size={20} />
          </PopoverTrigger>
          <PopoverContent side="top" align="end" className="max-h-[min(42rem,calc(100vh-1rem))] w-96 max-w-[calc(100vw-1rem)] overflow-y-auto">
            <PopoverHeader>
              <PopoverTitle>Controls</PopoverTitle>
              <PopoverDescription>The control style governs both camera and selection behavior.</PopoverDescription>
            </PopoverHeader>
            <div className="flex flex-col gap-3">
              <Field>
                <FieldLabel htmlFor="viewport-navigation-profile">Control style</FieldLabel>
                <Select
                  items={profileItems}
                  value={navigationProfileId}
                  onValueChange={(value) => {
                    if (isViewportNavigationProfileId(value)) onNavigationProfileId(value);
                  }}
                >
                  <SelectTrigger id="viewport-navigation-profile" className="w-full" aria-label="Control style">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent alignItemWithTrigger={false}>
                    <SelectGroup>
                      <SelectLabel>Camera and selection</SelectLabel>
                      {VIEWPORT_NAVIGATION_PROFILES.map((item) => (
                        <SelectItem key={item.id} value={item.id}>{item.label}</SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </Field>

              {navigationProfileId === 'custom' ? (
                <div className="flex flex-col gap-2">
                  <p className="font-medium">Custom mouse buttons</p>
                  {(['left', 'middle', 'right'] as const).map((button) => (
                    <CustomButtonField
                      key={button}
                      button={button}
                      handedness={mouseHandedness}
                      value={customNavigationSettings[button]}
                      onValue={(value) => onCustomNavigationSettings({ ...customNavigationSettings, [button]: value })}
                    />
                  ))}
                  <p className="text-xs text-muted-foreground">The wheel always zooms.</p>
                  <Separator />
                  <CustomSelectionFields settings={customSelectionSettings} onSettings={onCustomSelectionSettings} />
                </div>
              ) : null}

              <div className="flex flex-col gap-1.5">
                <ControlRow label="Rotate" value={rotateLabel} />
                <ControlRow label="Pan" value={panLabel} />
                <ControlRow label="Zoom" value={zoomLabel} />
              </div>

              <Separator />
              <div className="flex flex-col gap-1.5">
                <p className="font-medium">Selection</p>
                <ControlRow label="Pick" value={selection.pick} />
                <ControlRow label="Blank" value={selection.blank} />
                <ControlRow label="Window" value={selection.window} />
                <ControlRow label="Modify" value={selection.modify} />
              </div>

              {contextualShortcuts.length ? (
                <>
                  <Separator />
                  <div className="flex flex-col gap-1.5">
                    {contextualShortcuts.map((shortcut) => <ShortcutRow key={shortcut.id} shortcut={shortcut} />)}
                  </div>
                </>
              ) : null}

              <Separator />

              <Field>
                <FieldLabel>Mouse hand</FieldLabel>
                <ToggleGroup
                  aria-label="Mouse hand"
                  className="w-full"
                  value={[mouseHandedness]}
                  onValueChange={(value) => {
                    const nextHandedness = value[0];
                    if (isViewportMouseHandedness(nextHandedness)) onMouseHandedness(nextHandedness);
                  }}
                  size="sm"
                >
                  <ToggleGroupItem aria-label="Left-handed mouse" className="flex-1" value="left">Left-handed</ToggleGroupItem>
                  <ToggleGroupItem aria-label="Right-handed mouse" className="flex-1" value="right">Right-handed</ToggleGroupItem>
                </ToggleGroup>
              </Field>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}
