// Test-only Marionette framing. Sends each command once and matches its response ID.
import { createConnection } from "node:net";
import { pageWorldScript } from "./page-world.mjs";
export class MarionetteError extends Error {
  constructor(error) { super(error.message || error.error); this.response = error; }
}
export class MarionetteClient {
  constructor(socket, record) {
    this.socket = socket; this.record = record; this.buffer = Buffer.alloc(0);
    this.pending = new Map(); this.nextId = 0;
    this.hello = new Promise((resolve, reject) => { this.resolveHello = resolve; this.rejectHello = reject; });
    socket.on("data", data => {
      this.buffer = Buffer.concat([this.buffer, data]);
      while (true) {
        const separator = this.buffer.indexOf(58);
        if (separator < 0) break;
        const prefix = this.buffer.subarray(0, separator).toString();
        const length = Number(prefix);
        if (!/^\d+$/.test(prefix) || !Number.isSafeInteger(length) || length > 16 * 1024 * 1024) {
          this.fail(new Error("Invalid Marionette frame length")); socket.destroy(); return;
        }
        if (this.buffer.length < separator + 1 + length) break;
        let message;
        try { message = JSON.parse(this.buffer.subarray(separator + 1, separator + 1 + length).toString()); }
        catch (error) { this.fail(error); socket.destroy(); return; }
        this.buffer = this.buffer.subarray(separator + 1 + length);
        if (!Array.isArray(message)) { this.resolveHello(message); continue; }
        const [type, id, error, result] = message;
        const item = this.pending.get(id);
        if (!item) continue;
        this.pending.delete(id); clearTimeout(item.timer);
        this.record({ direction: "response", protocol: "marionette", id, type, error: error?.error });
        if (type !== 1) item.reject(new Error("Unexpected Marionette response"));
        else if (error) item.reject(new MarionetteError(error));
        else item.resolve(result);
      }
    });
    socket.on("error", error => this.fail(error));
    socket.on("close", () => this.fail(new Error("Marionette disconnected; no replay")));
  }
  static async connect(port, record) {
    const socket = createConnection({ host: "127.0.0.1", port });
    const client = new MarionetteClient(socket, record);
    const timer = setTimeout(() => socket.destroy(new Error("Marionette handshake deadline")), 10000);
    try {
      client.greeting = await client.hello;
      if (client.greeting.marionetteProtocol !== 3) throw new Error("Unexpected Marionette protocol version");
      return client;
    } finally { clearTimeout(timer); }
  }
  fail(error) {
    this.rejectHello(error);
    for (const item of this.pending.values()) { clearTimeout(item.timer); item.reject(error); }
    this.pending.clear();
  }
  command(name, parameters = {}, timeout = 10000) {
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id); reject(new Error(name + " deadline; execution may be unknown, no replay"));
      }, timeout);
      this.pending.set(id, { resolve, reject, timer });
      this.record({ direction: "request", protocol: "marionette", id, method: name });
      const payload = Buffer.from(JSON.stringify([0, id, name, parameters]));
      this.socket.write(Buffer.concat([Buffer.from(payload.length + ":"), payload]));
    });
  }
  async evaluate(expression) {
    const result = await this.command("WebDriver:ExecuteScript", {
      script: "return (async()=>JSON.stringify(await (" + expression + ")))();",
      args: [], sandbox: null, newSandbox: false,
    });
    return JSON.parse(result.value);
  }
  async evaluatePage(expression) {
    const result = await this.command("WebDriver:ExecuteScript", {
      script: pageWorldScript(expression),
      args: [], sandbox: "system", newSandbox: true,
    });
    return JSON.parse(result.value);
  }
  close() { this.socket.destroy(); }
}
