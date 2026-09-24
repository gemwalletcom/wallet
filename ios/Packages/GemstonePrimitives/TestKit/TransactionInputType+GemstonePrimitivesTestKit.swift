// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public extension TransactionInputType {
    static func transfer(_ asset: Primitives.Asset) -> Self {
        .transfer(asset: asset.toGem())
    }

    static func deposit(_ asset: Primitives.Asset) -> Self {
        .deposit(asset: asset.toGem())
    }

    static func swap(_ fromAsset: Primitives.Asset, _ toAsset: Primitives.Asset, _ swapData: Gemstone.SwapData) -> Self {
        .swap(fromAsset: fromAsset.toGem(), toAsset: toAsset.toGem(), swapData: swapData)
    }

    static func tokenApprove(_ asset: Primitives.Asset, _ approvalData: Gemstone.ApprovalData) -> Self {
        .tokenApprove(asset: asset.toGem(), approvalData: approvalData)
    }

    static func perpetual(_ asset: Primitives.Asset, _ perpetualType: Gemstone.PerpetualType) -> Self {
        .perpetual(asset: asset.toGem(), perpetualType: perpetualType)
    }

    static func generic(asset: Primitives.Asset, metadata: Primitives.ApplicationMetadata, extra: TransferDataExtra) -> Self {
        .generic(asset: asset.toGem(), metadata: metadata.toGem(), extra: extra)
    }
}
