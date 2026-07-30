// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum TransferAmountError: Error, Equatable, Sendable {
    case insufficientBalance(assetId: AssetId, requirement: BalanceRequirement)
    case insufficientNetworkFee(assetId: AssetId, requirement: BalanceRequirement)
    case minimumAccountBalanceTooLow(assetId: AssetId, requirement: BalanceRequirement)
}
