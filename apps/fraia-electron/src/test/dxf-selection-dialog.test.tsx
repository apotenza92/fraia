import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { DxfSelectionDialog } from '@/components/sources/DxfSelectionDialog';

const indexed = {
  resumed: false,
  derivative: { id: 'dxf-index' },
  index: {
    source_id: 'source-dxf',
    source_sha256: 'dxf-hash',
    model_space_name: 'Model',
    paper_layouts: [],
    units: undefined,
    layers: {
      FRAME: { name: 'FRAME', frozen: false, hidden: false, locked: false },
      'HIDDEN-GUIDES': { name: 'HIDDEN-GUIDES', frozen: true, hidden: true, locked: false },
    },
    blocks: {},
    entities: {
      'handle-10': { id: 'handle-10', entity_type: 'LINE', layer: 'FRAME', layout: 'Model', hidden: false, frozen: false },
      'handle-11': { id: 'handle-11', entity_type: 'LINE', layer: 'HIDDEN-GUIDES', layout: 'Model', hidden: true, frozen: true },
    },
    diagnostics: [{ code: 'units_unknown', message: 'DXF insertion units are not declared.' }],
  },
};

describe('DxfSelectionDialog', () => {
  it('preserves drawing diagnostics and requires an explicit view relation before preparation', async () => {
    const user = userEvent.setup();
    const prepared = { shelf_item: { id: 'reference-dxf', label: 'Frame elevation' }, interpretation: { observations: { line: { confirmation: { status: 'unconfirmed' }, designGeometry: null } } } };
    const onPrepared = vi.fn();
    Object.assign(window, { fraia: { prepareDxfSelection: vi.fn().mockResolvedValue(prepared) } });

    render(<DxfSelectionDialog open projectDir="/project" designId="design-a" designName="Frame" source={{ label: 'small-frame.dxf' }} indexed={indexed} interpretationParentRevisionId="revision-plan" onOpenChange={vi.fn()} onPrepared={onPrepared} />);

    expect(screen.getAllByText('Frozen').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Hidden').length).toBeGreaterThan(0);
    expect(screen.getByText('DXF insertion units are not declared.')).toBeVisible();
    await user.click(screen.getByRole('checkbox', { name: 'Select layer FRAME' }));
    await user.click(screen.getByRole('tab', { name: 'Review' }));
    expect(screen.getByText('This file does not declare units. Confirm the scale before use.')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Add design reference' })).toBeDisabled();
    await user.clear(screen.getByRole('textbox', { name: 'Name' }));
    await user.type(screen.getByRole('textbox', { name: 'Name' }), 'Frame elevation');
    await user.click(screen.getByRole('button', { name: 'Placement details' }));
    await user.clear(screen.getByRole('textbox', { name: 'Origin X' }));
    await user.type(screen.getByRole('textbox', { name: 'Origin X' }), '3.5');
    await user.click(screen.getByRole('checkbox', { name: /I checked the drawing view and scale/ }));
    await user.click(screen.getByRole('button', { name: 'Add design reference' }));

    await waitFor(() => expect(window.fraia.prepareDxfSelection).toHaveBeenCalledWith(expect.objectContaining({
      projectDir: '/project',
      designId: 'design-a',
      selection: expect.objectContaining({
        source_id: 'source-dxf',
        layout: 'Model',
        entity_ids: [],
        layer_names: ['FRAME'],
        block_names: [],
        view_role: 'plan',
        interpretation_parent_revision_id: 'revision-plan',
        relation_to_design: expect.objectContaining({
          confirmed: true,
          confirmed_by: 'user',
          transform: expect.objectContaining({ translation: [3.5, 0, 0], scale: [1, 1, 1] }),
          orientation: { forward: [0, 1, 0], up: [0, 0, 1] },
          scale: 1,
        }),
      }),
    })));
    expect(onPrepared).toHaveBeenCalledWith(prepared);
  });
});
