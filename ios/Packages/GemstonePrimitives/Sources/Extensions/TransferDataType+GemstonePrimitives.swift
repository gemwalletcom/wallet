// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransferDataType {
    var asset: Asset {
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
             let .generic(asset, _, _):
            asset
        case let .transferNft(asset):
            asset.chain.asset
        }
    }

    var feeAsset: Asset {
        let asset = asset
        if case .perpetual = self, asset.chain == .hyperCore {
            return Chain.hyperCore.defaultAsset(type: .perpetual)
        }
        return switch asset.chain {
        case .tempo: asset
        case .hyperCore: Chain.hyperCore.defaultAsset(type: .token)
        default:
            switch asset.id.type {
            case .native: asset
            case .token: asset.chain.asset
            }
        }
    }
}
