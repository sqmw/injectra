import { readdirSync, readFileSync, existsSync } from "node:fs";
import { dirname, resolve, relative } from "node:path";
import { root } from "./test-env.mjs";

function walk(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory()
    ? walk(resolve(directory, entry.name)) : entry.name.endsWith(".md") ? [resolve(directory, entry.name)] : []);
}
const files = [...readdirSync(root, { withFileTypes:true }).filter(entry => entry.isFile() && entry.name.endsWith(".md")).map(entry => resolve(root, entry.name)), ...walk(resolve(root, "docs")),
  ...(existsSync(resolve(root, ".github")) ? walk(resolve(root, ".github")) : []),
  ...(existsSync(resolve(root, "references")) ? walk(resolve(root, "references")) : []) ];
const graph = new Map();
let count = 0;
for (const file of files) {
  const links = [];
  const body = readFileSync(file, "utf8").replace(/```[^\n]*\n[\s\S]*?```/g, "");
  for (const match of body.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)) {
    if (/^https?:/.test(match[1])) continue;
    const [location, anchor] = match[1].split("#");
    const target = location ? resolve(dirname(file), location) : file;
    if (!existsSync(target)) throw Error(`${relative(root, file)}: missing ${match[1]}`);
    if (anchor && !readFileSync(target, "utf8").includes(`id="${anchor}"`)) throw Error(`Missing explicit anchor ${match[1]}`);
    links.push(target);
    count++;
  }
  graph.set(file, links);
}
const visited = new Set();
function visit(file) { if (visited.has(file)) return; visited.add(file); for (const next of graph.get(file) ?? []) visit(next); }
visit(resolve(root, "AGENTS.md"));
for (const file of files) if (!visited.has(file)) throw Error(`Unreachable document: ${relative(root, file)}`);
console.log(`Docs OK: ${files.length} Markdown files, ${count} local links, all reachable.`);
