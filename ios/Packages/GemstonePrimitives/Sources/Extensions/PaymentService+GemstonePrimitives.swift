// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension PaymentService {
    func load(link: PaymentLink, addresses: [Primitives.ChainAddress]) async throws -> TransferData {
        let transaction = try await load(
            link: link.map(),
            addresses: addresses.map { $0.map() },
        )
        return try transaction.map()
    }
}

private extension GemPaymentTransaction {
    func map() throws -> TransferData {
        let chain = try account.chain.map()
        return TransferData(
            asset: chain.asset,
            metadata: merchant.map(),
            transaction: transaction,
            memo: memo,
            outputType: .encodedTransaction,
            outputAction: .send,
            transactionType: transactionType.map(),
        )
    }
}

private extension Primitives.ChainAddress {
    func map() -> Gemstone.ChainAddress {
        Gemstone.ChainAddress(chain: chain.map(), address: address)
    }
}
