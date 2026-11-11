// Controller evidence is paired with the independently observed native app window.
export function evaluateProof(events, { path, nonce }) {
  const matches = event => event.path === path && event.nonce === nonce && event.peer === "127.0.0.1";
  const loaded = events.findIndex(event => matches(event) && event.phase === "loaded");
  const applied = events.findIndex((event, index) => index > loaded && loaded >= 0 && matches(event) && event.phase === "applied"
    && event.status === "applied" && event.connected === "true" && event.appRootPresent === "true" && event.color === "rgb(53, 32, 107)");
  const clicked = events.findIndex((event, index) => index > applied && applied >= 0 && matches(event) && event.phase === "clicked" && event.connected === "true");
  return { passed: loaded >= 0 && applied >= 0 && clicked >= 0, stages: { loaded, applied, clicked } };
}
