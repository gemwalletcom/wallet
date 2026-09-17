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
        asset: Asset = .mockEthereum(),
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
                assets: [.mockEthereum(), .mockEthereumUSDT()],
                fromAddress: fromAddress,
                toAddress: toAddress,
            ),
        )
    }
}
