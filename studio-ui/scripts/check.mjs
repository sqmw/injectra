import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { root, tools, requireTool } from './env.mjs';
const ts = requireTool('typescript');
const configuration = ts.readConfigFile(resolve(root, 'tsconfig.json'), ts.sys.readFile);
if (configuration.error) throw Error(ts.flattenDiagnosticMessageText(configuration.error.messageText, '\n'));
const parsed = ts.parseJsonConfigFileContent(configuration.config, ts.sys, root, { baseUrl: root, paths: { '*': [resolve(tools, 'node_modules/*')] } });
const program = ts.createProgram(parsed.fileNames, parsed.options);
const diagnostics = [...parsed.errors, ...ts.getPreEmitDiagnostics(program)];
if (diagnostics.length) { console.error(ts.formatDiagnosticsWithColorAndContext(diagnostics, { getCanonicalFileName: value => value, getCurrentDirectory: () => root, getNewLine: () => '\n' })); process.exit(1); }
await import('./build.mjs');
const result = spawnSync(process.execPath, ['--test', resolve(root, 'tests/model.test.mjs'), resolve(root, 'tests/api.test.mjs'), resolve(root, 'tests/studio.test.mjs')], { stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
