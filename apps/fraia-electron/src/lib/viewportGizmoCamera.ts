import * as THREE from 'three';

const VIEW_CUBE_HALF_EXTENT = 1.8;

type ViewportGizmoCameraInternals = {
  _camera?: THREE.Camera;
};

/**
 * three-viewport-gizmo renders cube gizmos with its own perspective camera.
 * Replace that display-only camera so the cube reads as a true orthographic
 * orientation aid. This does not change the model viewport camera.
 */
export function installOrthographicViewCubeCamera(viewGizmo: object) {
  const internals = viewGizmo as ViewportGizmoCameraInternals;
  if (!internals._camera) {
    throw new Error('Viewport gizmo camera internals are unavailable');
  }

  const camera = new THREE.OrthographicCamera(
    -VIEW_CUBE_HALF_EXTENT,
    VIEW_CUBE_HALF_EXTENT,
    VIEW_CUBE_HALF_EXTENT,
    -VIEW_CUBE_HALF_EXTENT,
    5,
    10,
  );
  camera.position.set(0, 0, 7);
  camera.updateProjectionMatrix();
  internals._camera = camera;
  return camera;
}
