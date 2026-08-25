// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemTransactionLoadMetadata {
    func getSequence() throws -> UInt64 {
        switch self {
        case let .ton(_, _, sequence),
             let .cosmos(_, sequence, _),
             let .near(sequence, _),
             let .stellar(sequence, _),
             let .xrp(sequence, _),
             let .algorand(sequence, _, _),
             let .aptos(sequence, _),
             let .polkadot(sequence, _, _, _, _, _, _),
             let .evm(sequence, _, _):
            return sequence
        case .none, .bitcoin, .zcash, .cardano, .tron, .solana, .sui, .hyperliquid:
            throw AnyError("Sequence not available for this metadata type")
        }
    }

    func getBlockNumber() throws -> UInt64 {
        switch self {
        case let .polkadot(_, _, _, blockNumber, _, _, _),
             let .tron(blockNumber, _, _, _, _, _, _),
             let .xrp(_, blockNumber),
             let .cardano(_, blockNumber):
            return blockNumber
        default:
            throw AnyError("Block number not available for this metadata type")
        }
    }
}
