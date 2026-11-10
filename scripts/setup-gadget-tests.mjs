import {mkdir, copyFile} from "node:fs/promises";
import {spawnSync} from "node:child_process";
import {resolve} from "node:path";
import {cache,root} from "./test-env.mjs";
const target=resolve(cache,"gadget-test-tools");
await mkdir(target,{recursive:true});
for(const name of ["package.json","package-lock.json"])await copyFile(resolve(root,"tests/gadget",name),resolve(target,name));
const p=spawnSync("npm",["ci","--no-audit","--no-fund","--registry=https://registry.npmjs.org"],{cwd:target,stdio:"inherit"});
if(p.error)throw p.error;
if(p.status!==0)process.exit(p.status??1);
// Explicitly run the pinned Electron downloader when npm install scripts are disabled.
const install=spawnSync(process.execPath,[resolve(target,"node_modules/electron/install.js")],{cwd:resolve(target,"node_modules/electron"),stdio:"inherit"});
if(install.error)throw install.error;process.exitCode=install.status??1;
