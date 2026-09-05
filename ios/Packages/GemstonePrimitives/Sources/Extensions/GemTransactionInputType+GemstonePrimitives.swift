// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransactionInputType {
    var asset: Primitives.Asset {
        inputAsset().map()
    }

    var chain: Primitives.Chain {
        Primitives.Chain(core: chain())
    }

    var applicationMetadata: Primitives.ApplicationMetadata? {
        guard case let .generic(_, metadata, _) = self else { return nil }
        return metadata.map()
    }
}

public extension GemTransactionInputType {
    static func transfer(_ asset: Primitives.Asset) -> Self {
        .transfer(asset: asset.map())
    }

    static func deposit(_ asset: Primitives.Asset) -> Self {
        .deposit(asset: asset.map())
    }

    static func withdrawal(_ asset: Primitives.Asset) -> Self {
        .withdrawal(asset: asset.map())
    }

    static func transferNft(_ nftAsset: Primitives.NFTAsset) -> Self {
        .transferNft(asset: nftAsset.chain.asset.map(), nftAsset: nftAsset.map())
    }

    static func swap(_ fromAsset: Primitives.Asset, _ toAsset: Primitives.Asset, _ swapData: Primitives.SwapData) -> Self {
        .swap(fromAsset: fromAsset.map(), toAsset: toAsset.map(), swapData: swapData.json())
    }

    static func tokenApprove(_ asset: Primitives.Asset, _ approvalData: Primitives.ApprovalData) -> Self {
        .tokenApprove(asset: asset.map(), approvalData: approvalData.json())
    }

    static func stake(_ asset: Primitives.Asset, _ stakeType: Primitives.StakeType) -> Self {
        .stake(asset: asset.map(), stakeType: stakeType.json())
    }

    static func account(_ asset: Primitives.Asset, _ accountType: Primitives.AccountDataType) -> Self {
        .account(asset: asset.map(), accountType: accountType.map())
    }

    static func perpetual(_ asset: Primitives.Asset, _ perpetualType: Primitives.PerpetualType) -> Self {
        .perpetual(asset: asset.map(), perpetualType: perpetualType.json())
    }

    static func earn(_ asset: Primitives.Asset, _ earnType: Primitives.EarnType, _ data: Primitives.ContractCallData) -> Self {
        .earn(asset: asset.map(), earnType: earnType.json(), data: data.json())
    }

    static func generic(asset: Primitives.Asset, metadata: Primitives.ApplicationMetadata, extra: GemTransferDataExtra) -> Self {
        .generic(asset: asset.map(), metadata: metadata.map(), extra: extra)
    }
}
