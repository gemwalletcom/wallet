// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.paymentVerificationOutcome
import InfoSheet
import Localization

@Observable
@MainActor
public final class PaymentVerificationSceneViewModel {
    private static let messageHandlerName = "payDataCollectionComplete"
    private static let messageType = "type"

    var isPresentingInfoSheet: InfoSheetType?

    let url: URL

    private let onComplete: () -> Void
    private let onError: () -> Void

    public init(url: URL, onComplete: @escaping () -> Void, onError: @escaping () -> Void) {
        self.url = url
        self.onComplete = onComplete
        self.onError = onError
    }

    var title: String {
        Localized.Info.paymentVerificationTitle
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
        guard let type = payload[Self.messageType] as? String else { return }
        switch paymentVerificationOutcome(messageType: type) {
        case .complete: onComplete()
        case .error: onError()
        case .ignored: break
        }
    }
}
