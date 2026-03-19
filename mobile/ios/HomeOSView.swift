// HomeOS — iOS WASM wrapper
// PLAN 7.2.2: WKWebView hosts origin.olang.wasm + origin.html
// No App Store restrictions (WASM = interpreted, not JIT)

import SwiftUI
import WebKit

/// HomeOS view: WKWebView running origin.olang WASM binary.
struct HomeOSView: UIViewRepresentable {
    let onOutput: (String) -> Void

    func makeUIView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        config.userContentController.add(
            context.coordinator, name: "homeos"
        )
        // Allow file access for local WASM loading
        config.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")

        let webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = context.coordinator

        // Load origin.html from app bundle
        if let htmlURL = Bundle.main.url(forResource: "origin", withExtension: "html") {
            webView.loadFileURL(htmlURL, allowingReadAccessTo: htmlURL.deletingLastPathComponent())
        }

        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(onOutput: onOutput)
    }

    class Coordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
        let onOutput: (String) -> Void

        init(onOutput: @escaping (String) -> Void) {
            self.onOutput = onOutput
        }

        // Receive messages from JavaScript (host_write → Swift)
        func userContentController(
            _ controller: WKUserContentController,
            didReceive message: WKScriptMessage
        ) {
            if message.name == "homeos", let body = message.body as? [String: Any] {
                if let text = body["output"] as? String {
                    onOutput(text)
                }
            }
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            // Inject bridge: route host_write to Swift via postMessage
            let bridge = """
            window.homeOSBridge = {
                write: function(text) {
                    window.webkit.messageHandlers.homeos.postMessage({output: text});
                }
            };
            """
            webView.evaluateJavaScript(bridge, completionHandler: nil)
        }
    }
}

/// Main iOS app entry point
@main
struct HomeOSApp: App {
    @State private var output: [String] = []

    var body: some Scene {
        WindowGroup {
            VStack {
                // Output display
                ScrollView {
                    VStack(alignment: .leading, spacing: 2) {
                        ForEach(output, id: \.self) { line in
                            Text(line)
                                .font(.system(.body, design: .monospaced))
                                .foregroundColor(.primary)
                        }
                    }
                    .padding()
                }

                // Input field
                HStack {
                    Text("○ >")
                        .font(.system(.body, design: .monospaced))
                        .foregroundColor(.secondary)
                    TextField("", text: .constant(""))
                        .font(.system(.body, design: .monospaced))
                        .textFieldStyle(.roundedBorder)
                }
                .padding(.horizontal)
                .padding(.bottom)
            }
            .overlay(
                HomeOSView { text in
                    output.append(text)
                }
                .frame(width: 0, height: 0) // Hidden webview
            )
        }
    }
}
