// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Primitives.Payment {
    static func decode(_ string: String, paymentService: GemPaymentService) throws -> Primitives.Payment {
        try Primitives.Payment(paymentService.decodeUrl(string: string))
    }
}

public extension GemPaymentService {
    func load(link: Primitives.PaymentLink, addresses: [Primitives.ChainAddress]) async throws -> GemPaymentTransaction {
        try await load(
            link: link.json(),
            addresses: addresses.map { $0.json() },
        )
    }
}
