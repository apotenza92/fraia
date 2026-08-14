import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { FirstSaveDialog } from '@/components/project/FirstSaveDialog';

describe('FirstSaveDialog', () => {
  it('explains the two identities, validates both, and focuses the first invalid field', async () => {
    const user = userEvent.setup();
    const onContinue = vi.fn();
    render(
      <FirstSaveDialog
        open
        projectName=""
        designName=""
        pending={false}
        onOpenChange={vi.fn()}
        onContinue={onContinue}
      />,
    );

    expect(screen.getByText('The folder for shared files and designs.')).toBeVisible();
    expect(screen.getByText('This structural model and its conversation. Use a unique name.')).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Choose location' }));

    const projectInput = screen.getByRole('textbox', { name: 'Project name' });
    const designInput = screen.getByRole('textbox', { name: 'Design name' });
    expect(projectInput).toHaveFocus();
    expect(projectInput).toHaveAttribute('aria-invalid', 'true');
    expect(designInput).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByText('Enter a project name.')).toBeVisible();
    expect(screen.getByText('Enter a design name.')).toBeVisible();

    await user.type(projectInput, '  Workshop  ');
    await user.type(designInput, '  Main frame  ');
    await user.click(screen.getByRole('button', { name: 'Choose location' }));

    expect(onContinue).toHaveBeenCalledWith({
      projectName: 'Workshop',
      designName: 'Main frame',
    });
  });
});
