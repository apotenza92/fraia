import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ResourceLibrarySheet } from '@/components/sources/ResourceLibrarySheet';

const source = {
  id: `sha256-${'a'.repeat(64)}`,
  sha256: 'a'.repeat(64),
  byte_size: 4096,
  detected_media_type: 'pdf',
  media_type: 'application/pdf',
  imported_at: '2026-08-13T10:00:00Z',
  aliases: [{ display_name: 'architectural-set.pdf', added_at: '2026-08-13T10:00:00Z', provenance: { origin_kind: 'native_file_dialog', supplied_name: 'architectural-set.pdf' } }],
};

describe('ResourceLibrarySheet', () => {
  it('keeps project files distinct from the current design references and supports keyboard rename', async () => {
    const user = userEvent.setup();
    let shelfItems: Record<string, any> = { plan: { id: 'plan', label: 'Gravity plan', kind: 'pdf_crop', provenance: { created_by: 'user', method: 'pdf_rectangle_crop' }, source: { source_id: source.id, source_sha256: source.sha256 } } };
    Object.assign(window, { fraia: {
      listSources: vi.fn().mockResolvedValue({ sources: [source] }),
      listShelf: vi.fn().mockImplementation(async () => ({ items: shelfItems })),
      onSourceImportProgress: vi.fn().mockReturnValue(() => {}),
      inspectSource: vi.fn().mockResolvedValue({ source, derivatives: [{ id: 'page-1', kind: 'page_image', parser: 'pdfium', parser_version: '1', byte_size: 100, created_at: '2026-08-13T10:00:01Z' }] }),
      indexPdfSource: vi.fn().mockResolvedValue({
        indexDerivative: { id: 'pdf-index', kind: 'pdf_index', parser: 'pdfium', parser_version: '1', byte_size: 100, created_at: '2026-08-13T10:00:01Z' },
        resumed: false,
        index: {
          sourceId: source.id,
          sourceSha256: source.sha256,
          parser: 'pdfium',
          parserVersion: '1',
          pageCount: 1,
          diagnostics: [],
          pages: [{
            pageNumber: 1,
            mediaBox: { x0: 10, y0: 20, x1: 610, y1: 820 },
            cropBox: { x0: 30, y0: 40, x1: 590, y1: 800 },
            rotationDegrees: 90,
            userUnit: 2,
            coordinateSpace: 'pdf_user_space_points',
            widthPoints: 760,
            heightPoints: 560,
            classification: 'vector_text',
            extractionMethod: 'native',
            nativeTextCharacters: 12,
            vectorPathOperations: 3,
            embeddedImageCount: 0,
          }],
        },
      }),
      upsertShelfItem: vi.fn().mockImplementation(async ({ item }) => {
        shelfItems = { ...shelfItems, [item.id]: item };
        return { items: shelfItems };
      }),
      removeShelfItem: vi.fn(),
      removeSource: vi.fn(),
      importSource: vi.fn(),
    } });

    const { rerender } = render(<ResourceLibrarySheet open initialView="sources" projectDir="/project" projectId="project-a" projectName="House" designId="design-a" designName="Gravity" onOpenChange={vi.fn()} />);
    expect(await screen.findByRole('tab', { name: 'Project files' })).toBeVisible();
    expect(screen.getByRole('tab', { name: 'Design references' })).toBeVisible();

    await user.click(screen.getByRole('button', { name: 'Inspect architectural-set.pdf' }));
    await screen.findByTestId('source-provenance');
    expect(screen.queryByText(`SHA-256 ${'a'.repeat(64)}`)).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'File provenance' }));
    expect(screen.getByText(`SHA-256 ${'a'.repeat(64)}`, { exact: false })).toBeVisible();
    expect(screen.getByText(/760 × 560 points · rotation 90° · user unit 2/)).toBeVisible();
    expect(screen.queryByRole('button', { name: /Add page/ })).not.toBeInTheDocument();
    await user.click(screen.getByRole('tab', { name: 'Design references' }));
    expect(await screen.findByText('Gravity plan')).toBeVisible();

    await user.click(screen.getByRole('button', { name: 'Rename' }));
    const label = screen.getByRole('textbox', { name: 'Reference name' });
    await user.clear(label);
    await user.type(label, 'Gravity plan{Enter}');
    await waitFor(() => expect(window.fraia.upsertShelfItem).toHaveBeenLastCalledWith(expect.objectContaining({
      designId: 'design-a',
      item: expect.objectContaining({ label: 'Gravity plan' }),
    })));

    shelfItems = {};
    rerender(<ResourceLibrarySheet open initialView="shelf" projectDir="/project" projectId="project-a" projectName="House" designId="design-b" designName="Lateral" onOpenChange={vi.fn()} />);
    expect(await screen.findByText('No design references yet')).toBeVisible();
    expect(screen.getAllByText('Project files are shared. References are used only by this design.').length).toBeGreaterThan(0);
  });

  it('shows real import progress states and a reference-safe removal error', async () => {
    const user = userEvent.setup();
    let progressListener: ((progress: any) => void) | undefined;
    Object.assign(window, { fraia: {
      listSources: vi.fn().mockResolvedValue({ sources: [source] }),
      listShelf: vi.fn().mockResolvedValue({ items: {} }),
      onSourceImportProgress: vi.fn().mockImplementation((listener) => { progressListener = listener; return () => {}; }),
      importSource: vi.fn().mockImplementation(async () => {
        progressListener?.({ state: 'uploading' });
        await Promise.resolve();
        progressListener?.({ state: 'processing' });
        return { record: source, job: { status: 'completed' }, deduplicated: false };
      }),
      removeSource: vi.fn().mockRejectedValue(new Error('source is referenced by design-a Shelf item plan')),
      inspectSource: vi.fn(),
      indexPdfSource: vi.fn(),
      upsertShelfItem: vi.fn(),
      removeShelfItem: vi.fn(),
    } });
    render(<ResourceLibrarySheet open initialView="sources" projectDir="/project" projectId="project-a" projectName="House" designId="design-a" designName="Gravity" onOpenChange={vi.fn()} />);
    await screen.findByText('architectural-set.pdf');
    await user.click(screen.getByRole('button', { name: 'Add project file' }));
    await waitFor(() => expect(screen.getByTestId('source-import-status')).toHaveAttribute('data-state', 'done'));
    await user.click(screen.getByRole('button', { name: 'Remove architectural-set.pdf from project' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Could not remove the file. Remove its design references first.');
  });
});
