// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit

public extension Transaction {
    static func mock(
        transactionId: TransactionId = TransactionId(chain: .ethereum, hash: "1"),
        type: TransactionType = .transfer,
        state: TransactionState = .confirmed,
        assetId: AssetId = .mock(),
        metadata: AnyCodableValue? = nil,
        fee: String = "1",
        blockNumber: String = "1",
    ) -> Transaction {
        Transaction(
            id: transactionId,
            assetId: assetId,
            from: "from",
            to: "to",
            contract: nil,
            type: type,
            state: state,
            blockNumber: blockNumber,
            sequence: "1",
            fee: fee,
            feeAssetId: assetId,
            value: "100",
            memo: nil,
            direction: .outgoing,
            utxoInputs: [],
            utxoOutputs: [],
            metadata: metadata,
            createdAt: .now,
        )
    }
}
