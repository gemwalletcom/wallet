// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import SwiftUI

public struct PaymentDataCollectionScene: View {
    private static let handlerName = "payDataCollectionComplete"
    private static let completeType = "IC_COMPLETE"
    private static let messageType = "type"

    private let url: URL
    private let onComplete: () -> Void

    public init(url: URL, onComplete: @escaping () -> Void) {
        self.url = url
        self.onComplete = onComplete
    }

    public var body: some View {
        NavigationStack {
            WebView(
                url: url,
                allowedHost: url.host() ?? "",
                messageHandler: WebViewMessageHandler(name: Self.handlerName, onMessage: onMessage),
            )
            .navigationBarTitleDisplayMode(.inline)
        }
    }

    private func onMessage(_ payload: [String: Any]) {
        guard payload[Self.messageType] as? String == Self.completeType else { return }
        onComplete()
    }
}
