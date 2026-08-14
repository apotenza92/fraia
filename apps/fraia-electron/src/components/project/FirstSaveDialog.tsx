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

export type FirstSaveNames = {
  projectName: string;
  designName: string;
};

export function FirstSaveDialog({
  open,
  projectName,
  designName,
  pending,
  error,
  onOpenChange,
  onContinue,
}: {
  open: boolean;
  projectName: string;
  designName: string;
  pending: boolean;
  error?: string | null;
  onOpenChange: (open: boolean) => void;
  onContinue: (names: FirstSaveNames) => void;
}) {
  const [projectInput, setProjectInput] = useState(projectName);
  const [designInput, setDesignInput] = useState(designName);
  const [projectError, setProjectError] = useState<string | null>(null);
  const [designError, setDesignError] = useState<string | null>(null);
  const projectRef = useRef<HTMLInputElement>(null);
  const designRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    setProjectInput(projectName);
    setDesignInput(designName);
    setProjectError(null);
    setDesignError(null);
  }, [open, projectName, designName]);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const nextProjectName = projectInput.trim();
    const nextDesignName = designInput.trim();
    const nextProjectError = nextProjectName ? null : 'Enter a project name.';
    const nextDesignError = nextDesignName ? null : 'Enter a design name.';
    setProjectError(nextProjectError);
    setDesignError(nextDesignError);
    if (nextProjectError) {
      projectRef.current?.focus();
      return;
    }
    if (nextDesignError) {
      designRef.current?.focus();
      return;
    }
    onContinue({ projectName: nextProjectName, designName: nextDesignName });
  }

  return (
    <Dialog open={open} onOpenChange={(nextOpen) => { if (!pending) onOpenChange(nextOpen); }}>
      <DialogContent initialFocus={projectRef} data-testid="first-save-dialog">
        <form onSubmit={submit} noValidate>
          <DialogHeader>
            <DialogTitle>Name this project and design</DialogTitle>
            <DialogDescription>Give the project and this design clear names. You will choose the folder next.</DialogDescription>
          </DialogHeader>
          <FieldGroup className="py-4">
            <Field data-invalid={Boolean(projectError)}>
              <FieldLabel htmlFor="first-save-project-name">Project name</FieldLabel>
              <Input
                ref={projectRef}
                id="first-save-project-name"
                value={projectInput}
                required
                aria-invalid={Boolean(projectError)}
                aria-describedby="first-save-project-description first-save-project-error"
                onChange={(event) => { setProjectInput(event.target.value); setProjectError(null); }}
              />
              <FieldDescription id="first-save-project-description">The folder for shared files and designs.</FieldDescription>
              <FieldError id="first-save-project-error">{projectError}</FieldError>
            </Field>
            <Field data-invalid={Boolean(designError)}>
              <FieldLabel htmlFor="first-save-design-name">Design name</FieldLabel>
              <Input
                ref={designRef}
                id="first-save-design-name"
                value={designInput}
                required
                aria-invalid={Boolean(designError)}
                aria-describedby="first-save-design-description first-save-design-error"
                onChange={(event) => { setDesignInput(event.target.value); setDesignError(null); }}
              />
              <FieldDescription id="first-save-design-description">This structural model and its conversation. Use a unique name.</FieldDescription>
              <FieldError id="first-save-design-error">{designError}</FieldError>
            </Field>
            {error ? <FieldError>{error}</FieldError> : null}
          </FieldGroup>
          <DialogFooter>
            <Button type="button" variant="outline" disabled={pending} onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button type="submit" disabled={pending}>{pending ? 'Opening location…' : 'Choose location'}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
