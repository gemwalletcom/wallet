// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import WebKit

@Observable
@MainActor
final class WebSceneViewModel: NSObject, Sendable {
    let url: URL
    var isLoading: Bool = true

    init(url: URL) {
        self.url = url
    }
}

// MARK: - WKNavigationDelegate

extension WebSceneViewModel: WKNavigationDelegate {
    func webView(_: WKWebView, didStartProvisionalNavigation _: WKNavigation) {
        isLoading = true
    }

    func webView(_: WKWebView, didCommit _: WKNavigation!) {
        isLoading = false
    }

    func webView(_: WKWebView, didFail _: WKNavigation, withError _: Error) {
        isLoading = false
    }

    func webView(_: WKWebView, didFailProvisionalNavigation _: WKNavigation, withError _: Error) {
        isLoading = false
    }
}
