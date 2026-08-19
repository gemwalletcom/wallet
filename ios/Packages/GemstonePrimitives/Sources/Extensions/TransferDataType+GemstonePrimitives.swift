// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
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
        guard
            let input = try? map(),
            let asset = try? Gemstone.transactionInputFeeAsset(inputType: input).map()
        else {
            preconditionFailure("Invalid fee asset for \(self)")
        }
        return asset
    }
}
