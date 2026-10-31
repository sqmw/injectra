const { app, BrowserWindow } = require("electron");
const path = require("node:path");

// This fixture owns its process and profile. It never attaches to a user's application.
app.setPath("userData", process.env.INJECTRA_FIXTURE_PROFILE);
app.commandLine.appendSwitch("remote-debugging-port", process.env.INJECTRA_FIXTURE_PORT);
app.commandLine.appendSwitch("remote-debugging-address", "127.0.0.1");
app.whenReady().then(async () => {
  const window = new BrowserWindow({
    show: false,
    width: 800,
    height: 600,
    webPreferences: { contextIsolation: true, nodeIntegration: false, sandbox: true },
  });
  await window.loadFile(path.join(__dirname, "index.html"));
});
process.on("SIGTERM", () => app.quit());
