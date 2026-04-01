// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public extension Transaction {
    static func id(chain: Chain, hash: String) -> String {
        String(format: "%@_%@", chain.rawValue, hash)
    }

    var chain: Chain {
        assetId.chain
    }

    var valueBigInt: BigInt {
        BigInt(value) ?? .zero
    }

    var feeBigInt: BigInt {
        BigInt(fee) ?? .zero
    }

    var assetIds: [AssetId] {
        switch type {
        case .transfer,
             .tokenApproval,
             .stakeDelegate,
             .stakeUndelegate,
             .stakeRedelegate,
             .stakeRewards,
             .stakeWithdraw,
             .assetActivation,
             .transferNFT,
             .smartContractCall,
             .perpetualOpenPosition,
             .perpetualClosePosition,
             .perpetualModifyPosition,
             .stakeFreeze,
             .stakeUnfreeze,
             .earnDeposit,
             .earnWithdraw:
            return [assetId]
        case .swap:
            guard let swapMetadata = metadata?.decode(TransactionSwapMetadata.self) else {
                return []
            }
            return [swapMetadata.fromAsset, swapMetadata.toAsset]
        }
    }

    var swapProvider: String? {
        metadata?.decode(TransactionSwapMetadata.self)?.provider
    }
}

extension Transaction: Identifiable {}
