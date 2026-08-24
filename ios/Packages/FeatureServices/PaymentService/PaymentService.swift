// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemPaymentService
import protocol Gemstone.GemPaymentServiceProtocol
import struct Gemstone.GemWalletConnectPayAuth
import GemstonePrimitives
import NativeProviderService
import Primitives

public protocol PaymentServiceable: Sendable {
    func getOptions(link: PaymentLink, addresses: [ChainAddress]) async throws -> PaymentOptions
    func getQuoteData(quote: PaymentQuote, addresses: [ChainAddress]) async throws -> PaymentQuoteData
    func confirm(payment: PaymentData, transactionHash: String) async throws -> PaymentOutcome
}

public final class PaymentService: Sendable {
    private let service: any GemPaymentServiceProtocol

    public init(service: any GemPaymentServiceProtocol) {
        self.service = service
    }

    public convenience init(
        nodeProvider: any NodeURLFetchable,
        auth: GemWalletConnectPayAuth,
        requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor(),
    ) {
        self.init(
            service: GemPaymentService(
                provider: NativeProvider(nodeProvider: nodeProvider, requestInterceptor: requestInterceptor),
                walletConnectPay: auth,
            ),
        )
    }
}

// MARK: - PaymentServiceable

extension PaymentService: PaymentServiceable {
    public func getOptions(link: PaymentLink, addresses: [ChainAddress]) async throws -> PaymentOptions {
        try await service.getOptions(link: link.map(), addresses: addresses.map { $0.map() }).map()
    }

    public func getQuoteData(quote: PaymentQuote, addresses: [ChainAddress]) async throws -> PaymentQuoteData {
        try await service.getQuoteData(quote: quote.map(), addresses: addresses.map { $0.map() }).map()
    }

    public func confirm(payment: PaymentData, transactionHash: String) async throws -> PaymentOutcome {
        try await service.confirm(quote: payment.quote.map(), transactionHash: transactionHash).map()
    }
}
