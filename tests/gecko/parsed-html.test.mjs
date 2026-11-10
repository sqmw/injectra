import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { requireTool, root } from "../../scripts/test-env.mjs";
import { parsedHtmlProgram } from "./parsed-html.mjs";
const { JSDOM } = requireTool("jsdom");
const runtime = readFileSync(join(root,"web/runtime.js"),"utf8");

test("HTML preparation preserves controls, escaped text and SVG namespace through shared runtime", async () => {
  const dom = new JSDOM("<!doctype html><html><head></head><body></body></html>", { runScripts:"outside-only" });
  const request = {operation:"apply", id:"test.parsed", css:"button{color:red}",
    html:'<button data-v="a&amp;b">&lt;plain&gt;</button><input value="value"><svg><circle r="2"/></svg>'};
  try {
    await dom.window.eval(parsedHtmlProgram(runtime,request,"context.root.querySelector('button').addEventListener('click',()=>globalThis.clicked=true);"));
    const button=dom.window.document.querySelector("button");button.click();
    assert.equal(dom.window.clicked,true);assert.equal(button.textContent,"<plain>");
    assert.equal(button.dataset.v,"a&b");assert.equal(dom.window.document.querySelector("input").value,"value");
    assert.equal(dom.window.document.querySelector("circle").namespaceURI,"http://www.w3.org/2000/svg");
    assert.equal(request.html.includes("<button"),true);
    assert.equal((await dom.window.eval(parsedHtmlProgram(runtime,{...request,html:"<button>updated</button>"}))).status,"updated");
    assert.equal(dom.window.document.querySelectorAll("[data-injectra]").length,2);
    assert.equal(dom.window.document.querySelector("button").textContent,"updated");
    await dom.window.eval(parsedHtmlProgram(runtime,{operation:"remove",id:request.id}));
    assert.equal(dom.window.document.querySelectorAll("[data-injectra]").length,0);
  } finally { dom.window.close(); }
});
test("fixture rejects script activation and shared runtime removes failed preparation nodes", async () => {
  const dom=new JSDOM("<!doctype html><html><head></head><body></body></html>",{runScripts:"outside-only"});
  try {
    await assert.rejects(dom.window.eval(parsedHtmlProgram(runtime,{operation:"apply",id:"test.script",css:"body{}",html:"<script>globalThis.unexpected=true</script>"})),/does not accept script/);
    assert.equal(dom.window.unexpected,undefined);
    assert.equal(dom.window.document.querySelectorAll("[data-injectra]").length,0);
  } finally { dom.window.close(); }
});

test("full shared contract updates parsed HTML and isolates a failed suite from the following suite", async () => {
  const { verifyRuntime } = await import("./runtime-checks.mjs");
  const dom=new JSDOM("<!doctype html><html><head></head><body></body></html>",{runScripts:"outside-only"});
  const second=new JSDOM("<!doctype html><html><head></head><body></body></html>",{runScripts:"outside-only"});
  const evaluate=async source=>JSON.parse(JSON.stringify(await dom.window.eval(source)));
  const other=async source=>JSON.parse(JSON.stringify(await second.window.eval(source)));
  try {
    await assert.rejects(verifyRuntime({evaluate,other,runtime,id:"test.interrupted",mountMode:"parsed-html",
      check:async(name, action)=>{if(name==="other target remains unchanged")throw Error("simulated assertion failure");await action();}}),/simulated assertion failure/);
    assert.equal(dom.window.document.querySelectorAll("[data-injectra]").length,0);
    assert.equal(dom.window.tbCleaned,undefined);
    for(const mountMode of ["parsed-html","dom"]) {
      await verifyRuntime({evaluate,other,runtime,id:"test."+mountMode,mountMode,check:async(name,action)=>action()});
      assert.equal(dom.window.document.querySelectorAll("[data-injectra]").length,0);
      assert.equal(dom.window.tbCleaned,undefined);
    }
  } finally {dom.window.close();second.window.close();}
});
