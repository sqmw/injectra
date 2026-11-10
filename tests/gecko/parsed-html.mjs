// Test-only HTML fixture preparation. It is not a product Bundle compiler.
// Keep the shared runtime's node ownership and use standard DOM construction.
export function parsedHtmlProgram(runtime, request, body = "") {
  const html = request.html || "";
  const prepared = { ...request, html: html ? " " : "" };
  const mount = html ? `
    const parsed = new DOMParser().parseFromString(${JSON.stringify(html)}, "text/html");
    if (parsed.querySelector("script")) throw Error("Parsed HTML fixture does not accept script elements; use the JS payload");
    context.root.replaceChildren(...Array.from(parsed.body.childNodes, node => document.importNode(node, true)));
  ` : "";
  return "(" + runtime + ")(" + JSON.stringify(prepared) + ",async context=>{" + mount + body + "})";
}
