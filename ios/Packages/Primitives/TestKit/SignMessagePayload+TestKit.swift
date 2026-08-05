// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension SignMessagePayload {
    static func mock(
        id: String = .empty,
        chain: Chain = .ethereum,
        appMetadata: TransactionAppMetadata = .mock(),
        wallet: Wallet = .mock(),
        message: SignMessage = SignMessage(chain: "ethereum", signType: .eip191, data: Data("test".utf8)),
        simulation: SimulationResult = .mock(),
        payment: PaymentData? = .none,
        expiresAt: Date? = .none,
    ) -> SignMessagePayload {
        SignMessagePayload(
            id: id,
            chain: chain,
            appMetadata: appMetadata,
            wallet: wallet,
            message: message,
            simulation: simulation,
            payment: payment,
            expiresAt: expiresAt,
        )
    }
}

public extension PaymentData {
    static func mock(
        provider: PaymentProviderName = .walletConnectPay,
        quote: PaymentQuote = .mock(),
        quotes: [PaymentQuote]? = .none,
        expiresAt: Date = Date(timeIntervalSinceNow: 900),
    ) -> PaymentData {
        PaymentData(
            provider: provider,
            quotes: .mock(expiresAt: expiresAt, quotes: quotes ?? [quote]),
            quote: quote,
        )
    }
}
