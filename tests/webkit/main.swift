import AppKit
import WebKit

final class Fixture: NSObject, NSApplicationDelegate, WKNavigationDelegate {
    var windows: [NSWindow] = []
    var completed: Set<ObjectIdentifier> = []
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        completed.insert(ObjectIdentifier(webView))
        if completed.count == 2, let path = ProcessInfo.processInfo.environment["INJECTRA_WK_FIXTURE_READY"] {
            try? Data("ready".utf8).write(to: URL(fileURLWithPath: path), options: .atomic)
        }
    }
    func applicationDidFinishLaunching(_ notification: Notification) {
        for name in ["main", "other"] {
            let configuration = WKWebViewConfiguration()
            configuration.websiteDataStore = .nonPersistent()
            let view = WKWebView(frame: NSRect(x: 0, y: 0, width: 800, height: 600), configuration: configuration)
            view.navigationDelegate = self
            let window = NSWindow(contentRect: view.frame, styleMask: [.titled, .closable], backing: .buffered, defer: false)
            window.isReleasedWhenClosed = false
            window.title = "Injectra WKWebView " + name
            window.contentView = view
            windows.append(window)
            view.loadHTMLString("<!doctype html><title>Injectra WKWebView \(name)</title><h1>Owned WebKit fixture</h1>", baseURL: URL(string: "https://injectra.invalid/\(name)"))
        }
    }
}
let app = NSApplication.shared
let delegate = Fixture()
app.delegate = delegate
app.setActivationPolicy(.prohibited)
app.run()
