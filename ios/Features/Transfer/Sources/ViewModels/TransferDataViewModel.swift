// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Localization
import Primitives
import PrimitivesComponents

struct TransferDataViewModel {
    let data: TransferData

    var type: TransferDataType {
        data.type
    }

    var recipientData: RecipientData {
        data.recipientData
    }

    var recipient: Recipient {
        recipientData.recipient
    }

    var asset: Asset {
        data.type.asset
    }

    var memo: String? {
        recipientData.recipient.memo
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
            switch type {
            case .stake: Localized.Transfer.Stake.title
            case .unstake: Localized.Transfer.Unstake.title
            case .redelegate: Localized.Transfer.Redelegate.title
            case .rewards: Localized.Transfer.ClaimRewards.title
            case .withdraw: Localized.Transfer.Withdraw.title
            case .freeze: Localized.Transfer.Freeze.title
            case .unfreeze: Localized.Transfer.Unfreeze.title
            }
        case let .account(_, type):
            switch type {
            case .activate: Localized.Transfer.ActivateAsset.title
            }
        case let .perpetual(_, type):
            switch type {
            case let .open(data): PerpetualDirectionViewModel(direction: data.direction).title
            case .close: Localized.Perpetual.closePosition
            case let .increase(data): PerpetualDirectionViewModel(direction: data.direction).increaseTitle
            case let .reduce(data): PerpetualDirectionViewModel(direction: data.positionDirection).reduceTitle
            case .modify: Localized.Perpetual.modifyPosition
            }
        case let .earn(_, type, _):
            switch type {
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
            URL(string: metadata.url)
        }
    }

    func availableValue(metadata: TransferDataMetadata?) -> BigInt {
        switch type {
        case .transfer,
             .deposit,
             .withdrawal,
             .swap,
             .tokenApprove,
             .generic,
             .transferNft,
             .perpetual: metadata?.available ?? .zero
        case let .account(_, type):
            switch type {
            case .activate: metadata?.available ?? .zero
            }
        case let .stake(_, stakeType):
            switch stakeType {
            case let .unstake(delegation): delegation.base.balanceValue
            case let .redelegate(data): data.delegation.base.balanceValue
            case let .withdraw(delegation): delegation.base.balanceValue
            case .rewards: data.value
            case .stake: metadata?.available ?? .zero
            case .freeze: metadata?.available ?? .zero
            case let .unfreeze(resource):
                switch resource {
                case .bandwidth: metadata?.assetBalance.frozen ?? .zero
                case .energy: metadata?.assetBalance.locked ?? .zero
                }
            }
        case let .earn(_, earnType, _):
            switch earnType {
            case .deposit: metadata?.available ?? .zero
            case let .withdraw(delegation): delegation.base.balanceValue
            }
        }
    }
}
