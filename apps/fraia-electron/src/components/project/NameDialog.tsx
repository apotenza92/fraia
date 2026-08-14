import { useEffect, useRef, useState, type FormEvent } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field';
import { Input } from '@/components/ui/input';

export function NameDialog({
  open,
  kind,
  initialValue,
  pending,
  error,
  onOpenChange,
  onSubmit,
}: {
  open: boolean;
  kind: 'create-design' | 'rename-design' | 'rename-project';
  initialValue: string;
  pending: boolean;
  error?: string | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (name: string) => void;
}) {
  const [value, setValue] = useState(initialValue);
  const [validation, setValidation] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const isProject = kind === 'rename-project';
  const title = kind === 'create-design' ? 'New design' : isProject ? 'Rename project' : 'Rename design';
  const label = isProject ? 'Project name' : 'Design name';

  useEffect(() => {
    if (!open) return;
    setValue(initialValue);
    setValidation(null);
  }, [initialValue, open]);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = value.trim();
    if (!name) {
      setValidation(`Enter a ${isProject ? 'project' : 'design'} name.`);
      inputRef.current?.focus();
      return;
    }
    onSubmit(name);
  }

  return (
    <Dialog open={open} onOpenChange={(nextOpen) => { if (!pending) onOpenChange(nextOpen); }}>
      <DialogContent initialFocus={inputRef} data-testid="name-dialog">
        <form onSubmit={submit} noValidate>
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
            <DialogDescription>
              {isProject
                ? 'Change the project display name without moving its saved folder.'
                : 'Design names identify their structural model, conversation, options, and analysis.'}
            </DialogDescription>
          </DialogHeader>
          <FieldGroup className="py-4">
            <Field data-invalid={Boolean(validation || error)}>
              <FieldLabel htmlFor="identity-name">{label}</FieldLabel>
              <Input
                ref={inputRef}
                id="identity-name"
                value={value}
                required
                aria-invalid={Boolean(validation || error)}
                onChange={(event) => { setValue(event.target.value); setValidation(null); }}
              />
              <FieldDescription>
                {isProject
                  ? 'Names this project, its shared files, and its designs.'
                  : 'Design names must be unique within this project.'}
              </FieldDescription>
              <FieldError>{validation ?? error}</FieldError>
            </Field>
          </FieldGroup>
          <DialogFooter>
            <Button type="button" variant="outline" disabled={pending} onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button type="submit" disabled={pending}>{pending ? 'Saving…' : kind === 'create-design' ? 'Create design' : 'Save name'}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
