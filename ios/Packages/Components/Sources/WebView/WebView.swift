// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import WebKit

public struct WebView: UIViewRepresentable {
    private let url: URL
    private let messageHandler: WebViewMessageHandler?
    private let allowedHost: String

    public init(
        url: URL,
        allowedHost: String,
        messageHandler: WebViewMessageHandler? = .none,
    ) {
        self.url = url
        self.allowedHost = allowedHost
        self.messageHandler = messageHandler
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(allowedHost: allowedHost, messageHandler: messageHandler)
    }

    public func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        if let messageHandler {
            configuration.userContentController.add(context.coordinator, name: messageHandler.name)
        }
        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.load(URLRequest(url: url))
        return webView
    }

    public func updateUIView(_: WKWebView, context _: Context) {}
}

public struct WebViewMessageHandler {
    public let name: String
    public let onMessage: ([String: Any]) -> Void

    public init(name: String, onMessage: @escaping ([String: Any]) -> Void) {
        self.name = name
        self.onMessage = onMessage
    }
}

public extension WebView {
    final class Coordinator: NSObject, WKScriptMessageHandler, WKNavigationDelegate {
        private let allowedHost: String
        private let messageHandler: WebViewMessageHandler?

        init(allowedHost: String, messageHandler: WebViewMessageHandler?) {
            self.allowedHost = allowedHost
            self.messageHandler = messageHandler
        }

        public func userContentController(_: WKUserContentController, didReceive message: WKScriptMessage) {
            guard let payload = Self.payload(from: message.body) else {
                return
            }
            messageHandler?.onMessage(payload)
        }

        public func webView(_: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping @MainActor (WKNavigationActionPolicy) -> Void) {
            guard let url = navigationAction.request.url, Self.isWeb(url: url) else {
                return decisionHandler(.cancel)
            }
            guard isAllowed(url: url) else {
                UIApplication.shared.open(url)
                return decisionHandler(.cancel)
            }
            decisionHandler(.allow)
        }

        private func isAllowed(url: URL) -> Bool {
            guard url.scheme == "https", let host = url.host() else {
                return false
            }
            return host == allowedHost || host.hasSuffix(".\(allowedHost)")
        }

        private static func isWeb(url: URL) -> Bool {
            ["https", "http"].contains(url.scheme ?? "")
        }

        private static func payload(from body: Any) -> [String: Any]? {
            if let payload = body as? [String: Any] {
                return payload
            }
            guard let text = body as? String, let data = text.data(using: .utf8) else {
                return .none
            }
            return try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        }
    }
}
