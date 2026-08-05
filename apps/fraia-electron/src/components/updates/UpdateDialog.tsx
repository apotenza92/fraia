import {
  CheckCircle2,
  CircleAlert,
  Download,
  PackageCheck,
  RefreshCw,
  ShieldCheck,
} from 'lucide-react';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Field, FieldContent, FieldDescription, FieldLabel } from '@/components/ui/field';
import { Progress, ProgressLabel, ProgressValue } from '@/components/ui/progress';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Spinner } from '@/components/ui/spinner';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  formatBytes,
  formatEta,
  formatLastChecked,
  UPDATE_FREQUENCY_LABELS,
  type UpdateFrequency,
  type UpdateStatus,
} from '@/lib/updateStatus';

const UPDATE_FREQUENCY_ITEMS = (Object.entries(UPDATE_FREQUENCY_LABELS) as Array<[UpdateFrequency, string]>)
  .map(([value, label]) => ({ value, label }));

type UpdateDialogProps = {
  checking: boolean;
  installing: boolean;
  onCheck: () => void;
  onInstall: () => void;
  onOpenChange: (open: boolean) => void;
  onSetFrequency: (frequency: UpdateFrequency) => void;
  open: boolean;
  status: UpdateStatus | null;
};

function UpdateState({ status }: { status: UpdateStatus }) {
  const progress = status.progress;

  if (status.phase === 'checking' || status.phase === 'initializing') {
    return (
      <Alert>
        <Spinner />
        <AlertTitle>{status.phase === 'initializing' ? 'Starting the updater' : 'Checking for updates'}</AlertTitle>
        <AlertDescription>
          Fraia is securely checking the {status.channel ?? 'current'} channel.
        </AlertDescription>
      </Alert>
    );
  }

  if (status.phase === 'available') {
    return (
      <Alert>
        <Download />
        <AlertTitle>Fraia {status.version} is available</AlertTitle>
        <AlertDescription>The update will download in the background.</AlertDescription>
      </Alert>
    );
  }

  if (status.phase === 'downloading' && progress) {
    return (
      <div className="flex flex-col gap-3" role="status" aria-live="polite">
        <Progress value={progress.percent}>
          <ProgressLabel>Downloading Fraia {status.version ?? 'update'}</ProgressLabel>
          <ProgressValue>{() => `${progress.percent}%`}</ProgressValue>
        </Progress>
        <div className="flex flex-wrap justify-between gap-2 text-xs text-muted-foreground">
          <span>
            {formatBytes(progress.transferred)}
            {progress.total ? ` of ${formatBytes(progress.total)}` : ''}
          </span>
          <span>
            {progress.bytesPerSecond ? `${formatBytes(progress.bytesPerSecond)}/s · ` : ''}
            {formatEta(progress.etaSeconds)}
          </span>
        </div>
      </div>
    );
  }

  if (status.phase === 'ready') {
    return status.releaseNotes ? (
      <div className="flex flex-col gap-2">
        <p className="text-sm font-medium">What&apos;s new</p>
        <ScrollArea
          aria-label={`Release notes for Fraia ${status.version ?? 'update'}`}
          className="h-44"
          role="region"
          tabIndex={0}
        >
          <p className="break-words whitespace-pre-wrap pr-3 text-sm">{status.releaseNotes}</p>
        </ScrollArea>
      </div>
    ) : null;
  }

  if (status.phase === 'installing') {
    return (
      <Alert>
        <Spinner />
        <AlertTitle>Installing the update</AlertTitle>
        <AlertDescription>Fraia will close and reopen when installation is complete.</AlertDescription>
      </Alert>
    );
  }

  if (status.phase === 'up-to-date') {
    return (
      <Alert>
        <CheckCircle2 />
        <AlertTitle>Fraia is up to date</AlertTitle>
        <AlertDescription>
          Version {status.currentVersion} is the newest release available on the {status.channel} channel.
        </AlertDescription>
      </Alert>
    );
  }

  if (status.phase === 'error') {
    return (
      <Alert variant="destructive">
        <CircleAlert />
        <AlertTitle>Fraia could not finish the update</AlertTitle>
        <AlertDescription>{status.errorMessage}</AlertDescription>
      </Alert>
    );
  }

  if (status.phase === 'managed') {
    return (
      <Alert>
        <PackageCheck />
        <AlertTitle>Updates are managed by your Linux package manager</AlertTitle>
        <AlertDescription>
          Update this DEB or RPM installation using the package manager that installed it. AppImage builds update themselves in Fraia.
        </AlertDescription>
      </Alert>
    );
  }

  if (status.phase === 'disabled') {
    return (
      <Alert>
        <CircleAlert />
        <AlertTitle>In-app updates are unavailable</AlertTitle>
        <AlertDescription>
          Update checking is available in packaged Fraia builds.
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <Alert>
      <ShieldCheck />
      <AlertTitle>Automatic updates are ready</AlertTitle>
      <AlertDescription>
        Fraia checks its own authenticated {status.channel} feed. This works whether Fraia was installed directly or through Homebrew.
      </AlertDescription>
    </Alert>
  );
}

export function UpdateDialog({
  checking,
  installing,
  onCheck,
  onInstall,
  onOpenChange,
  onSetFrequency,
  open,
  status,
}: UpdateDialogProps) {
  const ready = status?.phase === 'ready';
  const enabled = Boolean(status?.enabled);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg" showCloseButton={!installing}>
        <DialogHeader>
          <div className="flex flex-wrap items-center gap-2">
            <DialogTitle>{ready ? `Fraia ${status.version} is ready` : 'Fraia Updates'}</DialogTitle>
            {status?.channel ? (
              <Badge variant="secondary">{status.channel === 'beta' ? 'Beta channel' : 'Stable channel'}</Badge>
            ) : null}
          </div>
          <DialogDescription className={ready ? 'sr-only' : undefined}>
            {ready
              ? `Release notes and installation options for Fraia ${status.version}.`
              : `Current version ${status?.currentVersion ?? 'unknown'}.`}
          </DialogDescription>
        </DialogHeader>

        <div aria-live="polite">
          {status ? <UpdateState status={status} /> : (
            <Alert>
              <Spinner />
              <AlertTitle>Loading update status</AlertTitle>
              <AlertDescription>Fraia is reading the updater configuration.</AlertDescription>
            </Alert>
          )}
        </div>

        {enabled && !ready ? (
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel htmlFor="automatic-update-frequency">Automatic checks</FieldLabel>
              <FieldDescription>
                {formatLastChecked(status?.lastSuccessfulCheckAt)}
              </FieldDescription>
            </FieldContent>
            <Select
              items={UPDATE_FREQUENCY_ITEMS}
              value={status?.frequency ?? 'daily'}
              onValueChange={(value) => {
                if (typeof value === 'string' && value in UPDATE_FREQUENCY_LABELS) {
                  onSetFrequency(value as UpdateFrequency);
                }
              }}
            >
              <SelectTrigger id="automatic-update-frequency" aria-label="Automatic update frequency">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {(Object.entries(UPDATE_FREQUENCY_LABELS) as Array<[UpdateFrequency, string]>).map(([value, label]) => (
                    <SelectItem key={value} value={value}>{label}</SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>
        ) : null}

        <DialogFooter>
          {ready ? (
            <>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                Later
              </Button>
              <Button type="button" onClick={onInstall} disabled={installing}>
                {installing ? <Spinner data-icon="inline-start" /> : <RefreshCw data-icon="inline-start" />}
                Restart and update
              </Button>
            </>
          ) : enabled ? (
            <Button type="button" onClick={onCheck} disabled={checking || installing || status?.phase === 'downloading'}>
              {checking
                ? <Spinner data-icon="inline-start" />
                : <RefreshCw data-icon="inline-start" />}
              {checking ? 'Checking…' : status?.phase === 'downloading' ? 'Downloading…' : 'Check now'}
            </Button>
          ) : (
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Close</Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
