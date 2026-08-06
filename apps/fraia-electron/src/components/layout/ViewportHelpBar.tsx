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
  type ViewportNavigationProfile,
  type ViewportNavigationProfileId,
  type ViewportMouseHandedness,
} from '@/lib/viewportNavigation';

export type ViewportHelpShortcut = {
  id: string;
  keys: string[];
  label: string;
};

type MouseGesture = 'left' | 'middle' | 'right' | 'left+right' | 'wheel' | 'none';

function mouseGestureForAction(
  profile: ViewportNavigationProfile,
  action: Exclude<ViewportNavigationAction, 'none'>,
  handedness: ViewportMouseHandedness,
): MouseGesture {
  if (action === 'zoom') return 'wheel';
  const binding = profile.bindings.find((candidate) => candidate.action === action);
  if (!binding) return 'none';
  if (binding?.chord) return binding.chord;
  return handedMouseButton(binding?.button ?? 'left', handedness);
}

function MouseGestureIcon({ gesture, className }: { gesture: MouseGesture; className?: string }) {
  const leftActive = gesture === 'left' || gesture === 'left+right';
  const rightActive = gesture === 'right' || gesture === 'left+right';
  const middleActive = gesture === 'middle' || gesture === 'wheel';

  return (
    <svg
      aria-hidden="true"
      className={className}
      data-mouse-gesture={gesture}
      fill="none"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      {leftActive ? <path d="M5.75 8.75V8A5.25 5.25 0 0 1 11 2.75h.25v6Z" fill="currentColor" opacity="0.45" /> : null}
      {rightActive ? <path d="M12.75 2.75H13A5.25 5.25 0 0 1 18.25 8v.75h-5.5Z" fill="currentColor" opacity="0.45" /> : null}
      <rect height="20" rx="7" stroke="currentColor" strokeWidth="1.75" width="14" x="5" y="2" />
      <path d="M5.5 9h13M12 2.5V9" stroke="currentColor" strokeWidth="1.25" />
      {middleActive ? <rect fill="currentColor" height="5" rx="1.5" width="3" x="10.5" y="3.25" /> : null}
      {gesture === 'wheel' ? (
        <path d="m9.5 15 2.5-2.5 2.5 2.5M9.5 18l2.5 2.5 2.5-2.5" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.25" />
      ) : null}
    </svg>
  );
}

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

function Mapping({ label, value, gesture }: { label: string; value: string; gesture: MouseGesture }) {
  return (
    <Tooltip>
      <TooltipTrigger render={<span className="flex min-w-0 items-center gap-1.5" />}>
        <MouseGestureIcon gesture={gesture} className="size-3.5 shrink-0" />
        <span className="truncate"><span className="font-medium text-foreground">{label}</span> · {value}</span>
      </TooltipTrigger>
      <TooltipContent>{label}: {value}</TooltipContent>
    </Tooltip>
  );
}

function ControlRow({ label, value, gesture }: { label: string; value: string; gesture: MouseGesture }) {
  return (
    <div className="grid grid-cols-[1rem_4rem_1fr] items-center gap-2">
      <MouseGestureIcon gesture={gesture} className="size-4 text-muted-foreground" />
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
  mouseHandedness,
  contextualShortcuts,
  onNavigationProfileId,
  onCustomNavigationSettings,
  onMouseHandedness,
}: {
  availableWidth: number;
  status: string;
  navigationProfileId: ViewportNavigationProfileId;
  customNavigationSettings: ViewportCustomNavigationSettings;
  mouseHandedness: ViewportMouseHandedness;
  contextualShortcuts: ViewportHelpShortcut[];
  onNavigationProfileId: (profileId: ViewportNavigationProfileId) => void;
  onCustomNavigationSettings: (settings: ViewportCustomNavigationSettings) => void;
  onMouseHandedness: (handedness: ViewportMouseHandedness) => void;
}) {
  const profile = viewportNavigationProfile(navigationProfileId, customNavigationSettings);
  const rotateLabel = handedViewportNavigationLabel(profile.essentials.rotate, mouseHandedness);
  const panLabel = handedViewportNavigationLabel(profile.essentials.pan, mouseHandedness);
  const zoomLabel = handedViewportNavigationLabel(profile.essentials.zoom, mouseHandedness);
  const profileItems = VIEWPORT_NAVIGATION_PROFILES.map((item) => ({ value: item.id, label: item.label }));
  const showCameraMappings = availableWidth >= 520;

  return (
    <div data-testid="viewport-help-bar" className="h-9 shrink-0 bg-background">
      <Separator />
      <div className="flex h-[35px] min-w-0 items-center gap-3 px-2">
        <div data-testid="viewport-help-status" className="min-w-0 flex-1 truncate text-xs text-muted-foreground">
          {status}
        </div>

        {showCameraMappings ? (
          <div data-testid="viewport-help-essentials" className="flex min-w-0 items-center gap-3 text-xs text-muted-foreground">
            <Mapping label="Rotate" value={rotateLabel} gesture={mouseGestureForAction(profile, 'rotate', mouseHandedness)} />
            <Mapping label="Pan" value={panLabel} gesture={mouseGestureForAction(profile, 'pan', mouseHandedness)} />
            <Mapping label="Zoom" value={zoomLabel} gesture={mouseGestureForAction(profile, 'zoom', mouseHandedness)} />
          </div>
        ) : null}

        <Popover>
          <PopoverTrigger render={<Button variant="ghost" size="xs" className="shrink-0" />}>
            <CircleHelp data-icon="inline-start" />
            Controls
          </PopoverTrigger>
          <PopoverContent side="top" align="end" className="w-88 max-w-[calc(100vw-1rem)]">
            <PopoverHeader>
              <PopoverTitle>Controls</PopoverTitle>
              <PopoverDescription>Choose how the camera responds to your mouse.</PopoverDescription>
            </PopoverHeader>
            <div className="flex flex-col gap-3">
              <Field>
                <FieldLabel htmlFor="viewport-navigation-profile">Navigation profile</FieldLabel>
                <Select
                  items={profileItems}
                  value={navigationProfileId}
                  onValueChange={(value) => {
                    if (isViewportNavigationProfileId(value)) onNavigationProfileId(value);
                  }}
                >
                  <SelectTrigger id="viewport-navigation-profile" className="w-full" aria-label="Navigation profile">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent alignItemWithTrigger={false}>
                    <SelectGroup>
                      <SelectLabel>Camera navigation</SelectLabel>
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
                </div>
              ) : null}

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
                  variant="outline"
                  size="sm"
                  spacing={0}
                >
                  <ToggleGroupItem aria-label="Right-handed mouse" className="flex-1" value="right">Right-handed</ToggleGroupItem>
                  <ToggleGroupItem aria-label="Left-handed mouse" className="flex-1" value="left">Left-handed</ToggleGroupItem>
                </ToggleGroup>
              </Field>

              <div className="flex flex-col gap-1.5">
                <ControlRow label="Rotate" value={rotateLabel} gesture={mouseGestureForAction(profile, 'rotate', mouseHandedness)} />
                <ControlRow label="Pan" value={panLabel} gesture={mouseGestureForAction(profile, 'pan', mouseHandedness)} />
                <ControlRow label="Zoom" value={zoomLabel} gesture={mouseGestureForAction(profile, 'zoom', mouseHandedness)} />
              </div>

              <Separator />

              <div className="flex flex-col gap-2">
                <p className="font-medium">Selection</p>
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Kbd>Shift</Kbd><span>Keep selection when starting a window</span>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Kbd>Alt</Kbd><span>Drag a lasso</span>
                </div>
              </div>

              <Separator />

              <div className="flex flex-col gap-1.5">
                {contextualShortcuts.map((shortcut) => <ShortcutRow key={shortcut.id} shortcut={shortcut} />)}
                <ShortcutRow shortcut={{ id: 'delete', keys: ['Delete', '⌫'], label: 'Delete selection' }} />
                <ShortcutRow shortcut={{ id: 'escape', keys: ['Esc'], label: 'Cancel or clear' }} />
              </div>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}
