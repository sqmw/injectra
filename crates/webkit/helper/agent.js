import ObjC from 'frida-objc-bridge';
import { createAgent } from './webkit_agent.js';
import { createMacOSHost } from './macos_appkit.js';

// Explicit product assembly; unsupported profiles never fall back to this adapter.
rpc.exports = createAgent(ObjC, createMacOSHost(ObjC));
