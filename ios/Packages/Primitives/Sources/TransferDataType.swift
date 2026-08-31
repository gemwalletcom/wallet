// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum TransferDataType: Hashable, Equatable, Sendable {
    case transfer(Asset)
    case deposit(Asset)
    case withdrawal(Asset)
    case transferNft(NFTAsset)
    case swap(Asset, Asset, SwapData)
    case tokenApprove(Asset, ApprovalData)
    case stake(Asset, StakeType)
    case account(Asset, AccountDataType)
    case perpetual(Asset, PerpetualType)
    case earn(Asset, EarnType, ContractCallData)
    case generic(asset: Asset, metadata: ApplicationMetadata, extra: TransferDataExtra)

    public var applicationMetadata: ApplicationMetadata? {
        guard case let .generic(_, metadata, _) = self else { return nil }
        return metadata
    }

    public var chain: Chain {
        switch self {
        case let .transfer(asset),
             let .deposit(asset),
             let .withdrawal(asset),
             let .swap(asset, _, _),
             let .stake(asset, _),
             let .account(asset, _),
             let .perpetual(asset, _),
             let .earn(asset, _, _),
             let .tokenApprove(asset, _),
             let .generic(asset, _, _): asset.chain
        case let .transferNft(asset): asset.chain
        }
    }

    public var recentActivityData: RecentActivityData? {
        switch self {
        case let .transfer(asset): RecentActivityData(type: .transfer, assetId: asset.id, toAssetId: nil)
        case let .swap(from, to, _): RecentActivityData(type: .swap, assetId: from.id, toAssetId: to.id)
        default: nil
        }
    }
}
