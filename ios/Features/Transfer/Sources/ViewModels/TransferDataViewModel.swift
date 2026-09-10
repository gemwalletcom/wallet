// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmTitle
import enum Gemstone.TransactionInputType
import Foundation
import GemstonePrimitives
import Primitives
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
        data.title().title
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
             .earn,
             .payment: .none
        case let .generic(_, metadata, _):
            URL(string: metadata.toPrimitives().url)
        }
    }

}
