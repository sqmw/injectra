// Test-only WebDriver BiDi client. Product CLI does not use this module.
export class BidiError extends Error {
  constructor(response) {
    super(response.message ?? response.exceptionDetails?.text ?? response.error ?? "BiDi failure");
    this.response = response;
  }
}

export class BidiClient {
  constructor(socket, record) {
    this.socket = socket;
    this.record = record;
    this.nextId = 0;
    this.pending = new Map();
    socket.addEventListener("message", event => {
      let response;
      try { response = JSON.parse(String(event.data)); }
      catch (error) { this.rejectAll(error); socket.close(); return; }
      const waiting = this.pending.get(response.id);
      if (!waiting) { this.record({ direction: "unsolicited", id: response.id, type: response.type, method: response.method }); return; }
      clearTimeout(waiting.timer);
      this.pending.delete(response.id);
      this.record({ direction: "response", id: response.id, type: response.type,
        error: response.error, resultType: response.result?.type });
      if (response.type === "error") waiting.reject(new BidiError(response));
      else waiting.resolve(response.result);
    });
    socket.addEventListener("close", () => this.rejectAll(new Error("BiDi disconnected; no replay")));
    socket.addEventListener("error", () => this.rejectAll(new Error("BiDi socket failed; no replay")));
  }

  static async connect(url, record) {
    const socket = new WebSocket(url);
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        socket.close();
        reject(new Error("BiDi connection deadline"));
      }, 10000);
      socket.addEventListener("open", () => { clearTimeout(timer); resolve(); }, { once: true });
      socket.addEventListener("error", () => {
        clearTimeout(timer); reject(new Error("BiDi connection failed"));
      }, { once: true });
    });
    return new BidiClient(socket, record);
  }

  rejectAll(error) {
    for (const item of this.pending.values()) {
      clearTimeout(item.timer);
      item.reject(error);
    }
    this.pending.clear();
  }

  command(method, params = {}, timeout = 10000) {
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(method + " deadline; execution may be unknown, no replay"));
      }, timeout);
      this.pending.set(id, { resolve, reject, timer });
      this.record({ direction: "request", id, method });
      try { this.socket.send(JSON.stringify({ id, method, params })); }
      catch (error) {
        clearTimeout(timer); this.pending.delete(id); reject(error);
      }
    });
  }

  async evaluate(target, expression) {
    const result = await this.command("script.evaluate", {
      expression: "(async()=>JSON.stringify(await (" + expression + ")))()",
      target, awaitPromise: true, resultOwnership: "none",
    });
    if (result.type === "exception") throw new BidiError(result);
    if (result.type !== "success" || result.result?.type !== "string")
      throw new Error("Expected a serialized page result");
    return JSON.parse(result.result.value);
  }

  close() {
    this.rejectAll(new Error("Test client closed"));
    this.socket.close();
  }
}
