// Demo: custom code with the `three:ready` event.
// Click the cube in #custom to change its color.
document.addEventListener("three:ready", (event) => {
  if (event.target.id !== "custom") return;
  const { THREE, camera, root, renderer, requestRender } = event.detail;
  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();
  renderer.domElement.addEventListener("click", (e) => {
    const rect = renderer.domElement.getBoundingClientRect();
    pointer.set(((e.clientX - rect.left) / rect.width) * 2 - 1, -((e.clientY - rect.top) / rect.height) * 2 + 1);
    raycaster.setFromCamera(pointer, camera);
    const [hit] = raycaster.intersectObjects(root.children);
    if (hit) {
      hit.object.material.color.setHSL(Math.random(), 0.7, 0.6);
      requestRender();
    }
  });
});
