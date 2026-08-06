import { describe, expect, it } from 'vitest';
import * as THREE from 'three';

import { installOrthographicViewCubeCamera } from '@/lib/viewportGizmoCamera';

describe('viewport gizmo camera', () => {
  it('renders the view cube without perspective convergence', () => {
    const gizmo = { _camera: new THREE.PerspectiveCamera(26, 1, 5, 10) };

    const camera = installOrthographicViewCubeCamera(gizmo);
    camera.updateMatrixWorld(true);

    const nearPoint = new THREE.Vector3(1, 0, 1).project(camera);
    const farPoint = new THREE.Vector3(1, 0, -1).project(camera);

    expect(camera.isOrthographicCamera).toBe(true);
    expect(gizmo._camera).toBe(camera);
    expect(nearPoint.x).toBeCloseTo(farPoint.x);
    expect(nearPoint.y).toBeCloseTo(farPoint.y);
  });

  it('fails clearly if a dependency update removes the internal display camera', () => {
    expect(() => installOrthographicViewCubeCamera({})).toThrow(
      'Viewport gizmo camera internals are unavailable',
    );
  });
});
