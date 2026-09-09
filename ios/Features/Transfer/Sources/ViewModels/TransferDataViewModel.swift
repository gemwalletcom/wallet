// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmTitle
import enum Gemstone.TransactionInputType
import Foundation
import Localization
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData

struct TransferDataViewModel {
    let data: GemTransferData

    var type: TransactionInputType {
        data.inputType
    }

    var recipient: GemRecipient {
        data.recipient
    }

    var asset: Asset {
        data.asset
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
        switch data.title() {
        case .send: Localized.Transfer.Send.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Transfer.Withdraw.title
        case .swap: Localized.Wallet.swap
        case .approve: Localized.Transfer.Approve.title
        case .request: Localized.Transfer.reviewRequest
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        case .activateAsset: Localized.Transfer.ActivateAsset.title
        case let .perpetualOpen(direction): PerpetualDirectionViewModel(direction: direction.map()).title
        case let .perpetualIncrease(direction): PerpetualDirectionViewModel(direction: direction.map()).increaseTitle
        case let .perpetualReduce(direction): PerpetualDirectionViewModel(direction: direction.map()).reduceTitle
        case .perpetualClose: Localized.Perpetual.closePosition
        case .perpetualModify: Localized.Perpetual.modifyPosition
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
            URL(string: metadata.map().url)
        }
    }

}
