// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension TransactionViewModel {
    static func mock(
        type: TransactionType = .swap,
        state: TransactionState = .confirmed,
        direction: TransactionDirection = .incoming,
        from: String = "",
        to: String = "",
        fromAddress: AddressName? = nil,
        toAddress: AddressName? = nil,
        value: String = "1000000000000000000",
        asset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
        metadata: AnyCodableValue? = nil,
    ) -> TransactionViewModel {
        TransactionViewModel(
            transaction: .mock(
                transaction: .mock(
                    type: type,
                    state: state,
                    direction: direction,
                    from: from,
                    to: to,
                    value: value,
                    metadata: metadata,
                ),
                asset: asset,
                assets: [
                    .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
                    .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20),
                ],
                fromAddress: fromAddress,
                toAddress: toAddress,
            ),
        )
    }
}
