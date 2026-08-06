import { beforeEach, describe, expect, it } from 'vitest';

import {
  loadStoredViewportCamera,
  saveStoredViewportCamera,
  type StoredViewportCamera,
} from '@/lib/viewportCameraMemory';

const cameraA: StoredViewportCamera = {
  hasSceneGeometry: true,
  position: [12, 9, 12],
  target: [2, 3, 4],
  up: [0, 1, 0],
  zoom: 1.4,
  viewSize: 18,
};

describe('viewport camera memory', () => {
  beforeEach(() => window.localStorage.clear());

  it('keeps moved camera positions independent for each open file', () => {
    const cameraB: StoredViewportCamera = {
      ...cameraA,
      position: [-4, 20, 8],
      target: [0, 0, 0],
    };

    saveStoredViewportCamera('/projects/frame-a', cameraA);
    saveStoredViewportCamera('/projects/frame-b', cameraB);

    expect(loadStoredViewportCamera('/projects/frame-a')).toEqual(cameraA);
    expect(loadStoredViewportCamera('/projects/frame-b')).toEqual(cameraB);
    expect(loadStoredViewportCamera('/projects/new-frame')).toBeNull();
  });

  it('ignores malformed stored camera values', () => {
    window.localStorage.setItem('fraia:viewport-camera-memory-v1', JSON.stringify({
      '/projects/frame-a': { ...cameraA, zoom: 0 },
    }));

    expect(loadStoredViewportCamera('/projects/frame-a')).toBeNull();
  });
});
