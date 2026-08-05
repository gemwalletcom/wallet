// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import NativeProviderService
import Primitives

public protocol PaymentServiceable: PaymentStatusServiceable {
    func getPaymentOptions(link: PaymentLink, wallet: Wallet) async throws -> PaymentOptions
    func getPreparedPayment(provider: PaymentProviderName, quotes: PaymentQuotes, quote: PaymentQuote, wallet: Wallet) async throws -> PreparedPayment
    func confirmPayment(provider: PaymentProviderName, quote: PaymentQuote, actionResults: [String]) async throws -> PaymentOutcome
}

public final class PaymentService: PaymentServiceable {
    private let service: Gemstone.GemPaymentService

    public init(provider: NativeProvider, appId: String, clientId: String) {
        service = Gemstone.GemPaymentService(
            provider: provider,
            config: GemPaymentConfig(walletConnectPay: GemWalletConnectPayAuth(appId: appId, clientId: clientId)),
        )
    }

    public func getPaymentOptions(link: PaymentLink, wallet: Wallet) async throws -> PaymentOptions {
        try await service.getPaymentOptions(link: link.map(), addresses: wallet.chainAddresses.map { $0.map() }).map()
    }

    public func getPreparedPayment(provider: PaymentProviderName, quotes: PaymentQuotes, quote: PaymentQuote, wallet: Wallet) async throws -> PreparedPayment {
        let payment = try await service.getPreparedPayment(
            provider: provider.map(),
            quotes: quotes.map(),
            quote: quote.map(),
            addresses: wallet.chainAddresses.map { $0.map() },
        )
        return try PreparedPayment(
            quotes: payment.quotes.map(),
            quote: payment.quote.map(),
            actions: payment.actions.map { try $0.map() },
            isRelayed: payment.isRelayed,
        )
    }

    public func confirmPayment(provider: PaymentProviderName, quote: PaymentQuote, actionResults: [String]) async throws -> PaymentOutcome {
        try await service.confirmPayment(provider: provider.map(), quote: quote.map(), actionResults: actionResults).map()
    }

    public func hasStatus(provider: PaymentProviderName) -> Bool {
        Gemstone.paymentProviderHasStatus(provider: provider.map())
    }

    public func getPaymentStatus(provider: PaymentProviderName, paymentId: String) async throws -> PaymentOutcome {
        try await service.getPaymentStatus(provider: provider.map(), paymentId: paymentId).map()
    }
}
