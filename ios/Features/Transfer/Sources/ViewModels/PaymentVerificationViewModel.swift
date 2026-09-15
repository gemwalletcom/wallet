// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemPaymentServiceProtocol
import GemstonePrimitives
import Localization
import Primitives

@Observable
@MainActor
public final class PaymentVerificationViewModel {
    private static let messageHandlerName = "payDataCollectionComplete"
    private static let messageType = "type"
    private static let completeType = "IC_COMPLETE"

    public var isPresentingAlertMessage: AlertMessage?

    private let verification: PaymentVerification
    private let wallet: Wallet
    private let service: any GemPaymentServiceProtocol
    private let onComplete: (PaymentDestination) -> Void

    public init(
        verification: PaymentVerification,
        wallet: Wallet,
        service: any GemPaymentServiceProtocol,
        onComplete: @escaping (PaymentDestination) -> Void,
    ) {
        self.verification = verification
        self.wallet = wallet
        self.service = service
        self.onComplete = onComplete
    }

    var url: URL {
        verification.url
    }

    var allowedHost: String {
        verification.url.host() ?? ""
    }

    var messageHandler: WebViewMessageHandler {
        WebViewMessageHandler(name: Self.messageHandlerName) { [weak self] in self?.onMessage($0) }
    }
}

// MARK: - Business Logic

extension PaymentVerificationViewModel {
    func onMessage(_ payload: [String: Any]) {
        guard payload[Self.messageType] as? String == Self.completeType else { return }
        Task { await verified() }
    }

    func verified() async {
        do {
            let load = try await service.selectAsset(invoice: verification.invoice, addresses: wallet.chainAddresses, assetId: verification.assetId.identifier)
            onComplete(try PaymentDestination(load))
        } catch {
            isPresentingAlertMessage = AlertMessage(title: Localized.Errors.errorOccurred, message: error.localizedDescription)
        }
    }
}
