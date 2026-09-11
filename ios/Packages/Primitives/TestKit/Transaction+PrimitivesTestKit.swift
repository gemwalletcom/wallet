// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Transaction {
    static func mock(
        id: TransactionId = TransactionId(chain: .ethereum, hash: "1"),
        type: TransactionType = .transfer,
        state: TransactionState = .confirmed,
        direction: TransactionDirection = .incoming,
        assetId: AssetId = .mock(),
        from: String = "",
        to: String = "",
        value: String = "0",
        fee: String = "0",
        memo: String? = nil,
        metadata: AnyCodableValue? = nil,
    ) -> Transaction {
        Transaction(
            id: id,
            assetId: assetId,
            from: from,
            to: to,
            contract: .none,
            type: type,
            state: state,
            blockNumber: "",
            sequence: "",
            fee: fee,
            feeAssetId: assetId,
            value: value,
            memo: memo,
            direction: direction,
            utxoInputs: [],
            utxoOutputs: [],
            metadata: metadata,
            createdAt: .now,
        )
    }
}
