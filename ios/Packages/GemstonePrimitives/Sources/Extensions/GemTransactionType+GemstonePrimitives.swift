// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TransactionType {
    func map() -> Primitives.TransactionType {
        switch self {
        case .transfer: .transfer
        case .transferNft: .transferNFT
        case .swap: .swap
        case .tokenApproval: .tokenApproval
        case .stakeDelegate: .stakeDelegate
        case .stakeUndelegate: .stakeUndelegate
        case .stakeRewards: .stakeRewards
        case .stakeRedelegate: .stakeRedelegate
        case .stakeWithdraw: .stakeWithdraw
        case .stakeFreeze: .stakeFreeze
        case .stakeUnfreeze: .stakeUnfreeze
        case .assetActivation: .assetActivation
        case .smartContractCall: .smartContractCall
        case .perpetualOpenPosition: .perpetualOpenPosition
        case .perpetualClosePosition: .perpetualClosePosition
        case .perpetualModifyPosition: .perpetualModifyPosition
        case .earnDeposit: .earnDeposit
        case .earnWithdraw: .earnWithdraw
        }
    }
}
