const VIEWPORT_CAMERA_STORAGE_KEY = 'fraia:viewport-camera-memory-v1';

type VectorTuple = [number, number, number];

export type StoredViewportCamera = {
  hasSceneGeometry: boolean;
  position: VectorTuple;
  target: VectorTuple;
  up: VectorTuple;
  zoom: number;
  viewSize: number;
};

function finiteVector(value: unknown): value is VectorTuple {
  return Array.isArray(value)
    && value.length === 3
    && value.every((component) => typeof component === 'number' && Number.isFinite(component));
}

function validCamera(value: unknown): value is StoredViewportCamera {
  if (!value || typeof value !== 'object') return false;
  const camera = value as Partial<StoredViewportCamera>;
  return typeof camera.hasSceneGeometry === 'boolean'
    && finiteVector(camera.position)
    && finiteVector(camera.target)
    && finiteVector(camera.up)
    && typeof camera.zoom === 'number'
    && Number.isFinite(camera.zoom)
    && camera.zoom > 0
    && typeof camera.viewSize === 'number'
    && Number.isFinite(camera.viewSize)
    && camera.viewSize > 0;
}

function storedCameras(): Record<string, unknown> {
  try {
    const parsed = JSON.parse(window.localStorage.getItem(VIEWPORT_CAMERA_STORAGE_KEY) ?? '{}');
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

export function loadStoredViewportCamera(scopeKey: string): StoredViewportCamera | null {
  const camera = storedCameras()[scopeKey];
  return validCamera(camera) ? camera : null;
}

export function saveStoredViewportCamera(scopeKey: string, camera: StoredViewportCamera) {
  if (!validCamera(camera)) return;
  try {
    window.localStorage.setItem(VIEWPORT_CAMERA_STORAGE_KEY, JSON.stringify({
      ...storedCameras(),
      [scopeKey]: camera,
    }));
  } catch {
    // Camera memory is a convenience; storage restrictions must not break the viewport.
  }
}
