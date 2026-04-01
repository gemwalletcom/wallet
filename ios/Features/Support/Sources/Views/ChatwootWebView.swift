// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI
import WebKit

struct ChatwootWebView: UIViewRepresentable {
    let model: ChatwootWebViewModel

    func makeUIView(context _: Context) -> WKWebView {
        let configuration = model.configureWebView()

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = model
        webView.isOpaque = false
        webView.backgroundColor = .clear
        webView.scrollView.backgroundColor = .clear

        return webView
    }

    func updateUIView(_ webView: WKWebView, context _: Context) {
        webView.loadHTMLString(model.htmlContent, baseURL: model.baseUrl)
    }

    static func dismantleUIView(_ webView: WKWebView, coordinator _: ()) {
        webView.navigationDelegate = nil
        webView.configuration.userContentController.removeAllScriptMessageHandlers()
    }
}
