import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { DrawingInterpretationDialog } from '@/components/sources/DrawingInterpretationDialog';

describe('DrawingInterpretationDialog', () => {
  it('creates an exact unconfirmed crop observation and confirms it explicitly', async () => {
    const user = userEvent.setup();
    let head: any = null;
    Object.assign(window, { fraia: {
      listDrawingInterpretations: vi.fn().mockImplementation(async () => ({ projectId: 'project-a', designId: 'design-a', headRevisionId: head?.revisionId, revisions: [] })),
      inspectDrawingInterpretation: vi.fn().mockImplementation(async () => head),
      createDrawingInterpretation: vi.fn().mockImplementation(async (request) => {
        head = { schemaVersion: 'fraia.drawing-interpretation.v1', revisionId: 'revision-1', ...request.revision };
        return head;
      }),
      confirmDrawingObservations: vi.fn().mockImplementation(async (request) => {
        const id = request.operation.observationIds[0];
        head = { ...head, revisionId: 'revision-2', parentRevisionId: 'revision-1', observations: { ...head.observations, [id]: { ...head.observations[id], confirmation: { status: 'confirmed', confirmedBy: 'user', confirmedAt: request.operation.confirmedAt } } } };
        return head;
      }),
      reconcileDrawingInterpretation: vi.fn(),
      resolveDrawingInterpretationConflict: vi.fn(),
    } });
    render(<DrawingInterpretationDialog open projectDir="/project" projectId="project-a" designId="design-a" designName="Frame" reference={{ id: 'crop-1', label: 'Plan crop', kind: 'pdf_crop', source: { source_id: 'source-a', source_sha256: 'hash-a' }, page_number: 2, crop: { x: 10, y: 20, width: 30, height: 40, coordinate_space: 'pdf_user_space_points' }, drawing_context: { view_role: 'plan' } }} onOpenChange={vi.fn()} />);
    await user.click(await screen.findByRole('button', { name: 'Review this reference' }));
    await waitFor(() => expect(window.fraia.createDrawingInterpretation).toHaveBeenCalledWith(expect.objectContaining({ authority: 'user', revision: expect.objectContaining({ observations: expect.objectContaining({ 'observation-crop-1': expect.objectContaining({ sourceLocator: { locatorKind: 'pdf_page', page_number: 2, coordinate_space: 'pdf_user_space_points' }, sourceGeometry: { sourceGeometryKind: 'region', boundary: [[10, 20], [40, 20], [40, 60], [10, 60]] }, confirmation: { status: 'unconfirmed' } }) }) }) })));
    expect(screen.getByText('unconfirmed')).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Confirm' }));
    await waitFor(() => expect(window.fraia.confirmDrawingObservations).toHaveBeenCalledWith(expect.objectContaining({ operation: expect.objectContaining({ expectedParentRevisionId: 'revision-1', observationIds: ['observation-crop-1'] }) })));
    expect(screen.getByText('confirmed')).toBeVisible();
  });
});
