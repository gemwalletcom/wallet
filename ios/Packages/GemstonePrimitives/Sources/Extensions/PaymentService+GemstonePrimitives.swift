// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension PaymentService {
    func loadPaymentTransaction(link: PaymentLink, addresses: [Primitives.ChainAddress]) async throws -> GemPaymentTransaction {
        try await load(
            link: link.map(),
            addresses: addresses.map { $0.map() },
        )
    }
}

private extension Primitives.ChainAddress {
    func map() -> Gemstone.ChainAddress {
        Gemstone.ChainAddress(chain: chain.map(), address: address)
    }
}
