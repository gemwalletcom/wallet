// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension PaymentService {
    func load(link: Primitives.PaymentLink, addresses: [Primitives.ChainAddress]) async throws -> GemPaymentTransaction {
        try await load(
            link: link.json(),
            addresses: addresses.map { try $0.json() },
        )
    }
}
