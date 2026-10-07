// E2E fixture: a page module that runs after init.js and listens for
// three:ready. It must still get the event.
window.__lateReady = [];
document.addEventListener("three:ready", (event) => window.__lateReady.push(event.target.id));
