import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { IfcSelectionDialog } from '@/components/sources/IfcSelectionDialog';

const transform = { translation: [0, 0, 3000] as [number, number, number], rotation_degrees: [0, 0, 0] as [number, number, number], scale: [1, 1, 1] as [number, number, number] };
const indexed = { resumed: false, derivative: {}, index: { source_id: 'ifc-source', source_sha256: 'ifc-hash', file_schema: ['IFC4'], length_unit: 'mm', objects: { BEAM1: { step_id: 20, global_id: 'BEAM1', class_name: 'IFCBEAM', name: 'Architect beam', transform, storey_id: 10 } }, storeys: { '10': { step_id: 10, global_id: 'STOREY2', name: 'Level 2', elevation: 3000, transform } }, grids: {}, diagnostics: [{ code: 'unsupported_representation', step_id: 20, message: 'Representation #99 is unsupported.' }] } };

describe('IfcSelectionDialog', () => {
  it('filters exact model identities and prepares a read-only reference', async () => {
    const user = userEvent.setup();
    const prepared = { shelf_item: { id: 'ifc-ref', label: 'Level 2 reference' }, interpretation: { observations: {} } };
    Object.assign(window, { fraia: { prepareIfcSelection: vi.fn().mockResolvedValue(prepared) } });
    const onPrepared = vi.fn();
    render(<IfcSelectionDialog open projectDir="/project" designId="design-a" designName="Frame" sourceLabel="architect-reference.ifc" indexed={indexed} interpretationParentRevisionId="revision-a" onOpenChange={vi.fn()} onPrepared={onPrepared} />);
    expect(screen.getByText('Representation #99 is unsupported.')).toBeVisible();
    await user.click(screen.getByRole('checkbox', { name: 'Select storey Level 2' }));
    await user.click(screen.getByRole('button', { name: 'Reference details' }));
    await user.clear(screen.getByRole('textbox', { name: 'Name' }));
    await user.type(screen.getByRole('textbox', { name: 'Name' }), 'Level 2 reference');
    await user.click(screen.getByRole('button', { name: 'Add design reference' }));
    await waitFor(() => expect(window.fraia.prepareIfcSelection).toHaveBeenCalledWith(expect.objectContaining({ projectDir: '/project', designId: 'design-a', selection: expect.objectContaining({ source_id: 'ifc-source', storey_ids: [10], object_ids: [], grid_ids: [], class_names: [], interpretation_parent_revision_id: 'revision-a' }) })));
    expect(onPrepared).toHaveBeenCalledWith(prepared);
  });
});
