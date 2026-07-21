// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

extension TransferData {
    func availableValue(metadata: TransferDataMetadata) -> BigInt {
        switch type {
        case .transfer,
             .deposit,
             .withdrawal,
             .swap,
             .tokenApprove,
             .generic,
             .transferNft,
             .perpetual,
             .account(_, .activate),
             .stake(_, .stake),
             .stake(_, .freeze),
             .earn(_, .deposit, _): metadata.available
        case let .stake(_, .unstake(delegation)),
             let .stake(_, .withdraw(delegation)),
             let .earn(_, .withdraw(delegation), _): delegation.base.balanceValue
        case let .stake(_, .redelegate(data)): data.delegation.base.balanceValue
        case .stake(_, .rewards): value
        case .stake(_, .unfreeze(.bandwidth)): metadata.assetBalance.frozen
        case .stake(_, .unfreeze(.energy)): metadata.assetBalance.locked
        }
    }
}
