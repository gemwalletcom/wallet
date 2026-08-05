// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import PaymentService
import Primitives
import SwiftUI
import PrimitivesComponents

public struct PaymentDataCollectionScene: View {
    private static let messageHandlerName = "payDataCollectionComplete"
    private static let completeMessageType = "IC_COMPLETE"
    private static let errorMessageType = "IC_ERROR"
    private static let messageTypeKey = "type"
    private static let messageErrorKey = "error"

    private let callback: SheetCallback<PaymentDataCollectionRequest>
    private let onComplete: () -> Void

    public init(
        callback: SheetCallback<PaymentDataCollectionRequest>,
        onComplete: @escaping () -> Void,
    ) {
        self.callback = callback
        self.onComplete = onComplete
    }

    public var body: some View {
        WebView(
            url: callback.payload.url,
            allowedHost: callback.payload.url.host() ?? .empty,
            messageHandler: WebViewMessageHandler(name: Self.messageHandlerName, onMessage: onMessage),
        )
        .ignoresSafeArea(edges: .bottom)
    }

    private func onMessage(_ payload: [String: Any]) {
        switch payload[Self.messageTypeKey] as? String {
        case Self.completeMessageType:
            finish(.success(.empty))
        case Self.errorMessageType:
            finish(.failure(AnyError(payload[Self.messageErrorKey] as? String ?? Localized.Errors.transferError)))
        default:
            break
        }
    }

    private func finish(_ result: Result<String, any Error>) {
        callback.delegate(result)
        onComplete()
    }
}
