// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import Foundation
import Localization
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData

struct TransferDataViewModel {
    let data: GemTransferData

    var type: GemTransactionInputType {
        data.inputType
    }

    var recipient: GemRecipient {
        data.recipient
    }

    var asset: Asset {
        data.inputType.asset
    }

    var memo: String? {
        recipient.memo
    }

    var chain: Chain {
        data.chain
    }

    var chainType: ChainType {
        chain.type
    }

    var chainAsset: Asset {
        chain.asset
    }

    var title: String {
        switch type {
        case .transfer: Localized.Transfer.Send.title
        case .deposit: Localized.Wallet.deposit
        case .withdrawal: Localized.Wallet.withdraw
        case .transferNft: Localized.Transfer.Send.title
        case .swap, .tokenApprove: Localized.Wallet.swap
        case .generic: Localized.Transfer.reviewRequest
        case let .stake(_, type):
            switch Primitives.StakeType(core: type) {
            case .stake: Localized.Transfer.Stake.title
            case .unstake: Localized.Transfer.Unstake.title
            case .redelegate: Localized.Transfer.Redelegate.title
            case .rewards: Localized.Transfer.ClaimRewards.title
            case .withdraw: Localized.Transfer.Withdraw.title
            case .freeze: Localized.Transfer.Freeze.title
            case .unfreeze: Localized.Transfer.Unfreeze.title
            }
        case let .account(_, type):
            switch Primitives.AccountDataType(core: type) {
            case .activate: Localized.Transfer.ActivateAsset.title
            }
        case let .perpetual(_, type):
            switch Primitives.PerpetualType(core: type) {
            case let .open(data): PerpetualDirectionViewModel(direction: data.direction).title
            case .close: Localized.Perpetual.closePosition
            case let .increase(data): PerpetualDirectionViewModel(direction: data.direction).increaseTitle
            case let .reduce(data): PerpetualDirectionViewModel(direction: data.positionDirection).reduceTitle
            case .modify: Localized.Perpetual.modifyPosition
            }
        case let .earn(_, type, _):
            switch Primitives.EarnType(core: type) {
            case .deposit: Localized.Wallet.deposit
            case .withdraw: Localized.Transfer.Withdraw.title
            }
        }
    }

    var websiteURL: URL? {
        switch type {
        case .transfer,
             .deposit,
             .withdrawal,
             .transferNft,
             .swap,
             .tokenApprove,
             .stake,
             .account,
             .perpetual,
             .earn: .none
        case let .generic(_, metadata, _):
            URL(string: Primitives.ApplicationMetadata(core: metadata).url)
        }
    }

}
