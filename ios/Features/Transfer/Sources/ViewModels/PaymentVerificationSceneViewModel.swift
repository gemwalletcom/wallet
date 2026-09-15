// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import Localization

@Observable
@MainActor
public final class PaymentVerificationSceneViewModel {
    private static let messageHandlerName = "payDataCollectionComplete"
    private static let messageType = "type"
    private static let completeType = "IC_COMPLETE"

    public var isPresentingInfoSheet: InfoSheetType?

    let url: URL

    private let onComplete: () -> Void

    public init(url: URL, onComplete: @escaping () -> Void) {
        self.url = url
        self.onComplete = onComplete
    }

    var title: String {
        Localized.Info.paymentVerificationTitle
    }

    var allowedHost: String {
        url.host() ?? ""
    }

    var messageHandler: WebViewMessageHandler {
        WebViewMessageHandler(name: Self.messageHandlerName) { [weak self] in self?.onMessage($0) }
    }
}

// MARK: - Business Logic

extension PaymentVerificationSceneViewModel {
    func onSelectInfo() {
        isPresentingInfoSheet = .paymentVerification
    }

    func onMessage(_ payload: [String: Any]) {
        guard payload[Self.messageType] as? String == Self.completeType else { return }
        onComplete()
    }
}
