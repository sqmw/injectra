// Copy package files without retaining links into the installed application.
import fs from 'node:fs/promises';
import path from 'node:path';

export async function copyTree(source, destination, include = () => true) {
  source = await fs.realpath(source);
  destination = path.resolve(destination);
  const inside = (root, item) => {
    const relative = path.relative(root, item);
    return relative === '' || (!path.isAbsolute(relative) && relative !== '..' && !relative.startsWith('..' + path.sep));
  };
  if (inside(source, destination) || inside(destination, source)) throw Error('Copy roots must be disjoint');
  try { await fs.lstat(destination); throw Error('Destination already exists'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  const links = [];
  async function scan(directory) {
    for (const entry of await fs.readdir(directory, {withFileTypes:true})) {
      const item = path.join(directory, entry.name);
      if (!include(item)) continue;
      const stat = await fs.lstat(item);
      if (stat.isSymbolicLink()) {
        // Some installed Windows junctions carry a trailing separator that
        // libuv cannot stat through. Canonicalize their explicit target instead.
        const raw = await fs.readlink(item);
        const target = await fs.realpath(path.resolve(path.dirname(item), raw));
        if (!inside(source, target) || !include(target)) throw Error('Unsupported: link leaves copied package');
        const directory = (await fs.stat(target)).isDirectory();
        if (inside(target, path.dirname(item))) throw Error('Unsupported: cyclic directory link');
        links.push({item, target, directory});
      } else if (stat.isDirectory()) await scan(item);
    }
  }
  // Validate all links before writing a partial copy.
  await scan(source);
  const linkPaths = new Set(links.map(link => link.item));
  await fs.cp(source, destination, {recursive:true, dereference:false, filter:item => include(item) && !linkPaths.has(item)});
  for (const link of links) {
    const output = path.join(destination, path.relative(source, link.item));
    const target = path.join(destination, path.relative(source, link.target));
    // Real targets must have been copied, including their parent filter decisions.
    await fs.stat(target);
    await fs.symlink(process.platform === 'win32' ? target : path.relative(path.dirname(output), target), output,
      link.directory ? (process.platform === 'win32' ? 'junction' : 'dir') : 'file');
  }
  return links.map(link => ({path:path.relative(source, link.item), target:path.relative(source, link.target)}));
}
