// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import Components
import Primitives
import PrimitivesComponents
import struct Gemstone.GemRecipient

struct ConfirmMemoViewModel {
    private let type: GemTransactionInputType
    private let recipient: GemRecipient

    init(type: GemTransactionInputType, recipient: GemRecipient) {
        self.type = type
        self.recipient = recipient
    }
}

// MARK: - ItemModelProvidable

extension ConfirmMemoViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard showMemo else { return .empty }
        return .memo(MemoViewModel(memo: recipient.memo).listItemModel)
    }
}

// MARK: - Private

extension ConfirmMemoViewModel {
    private var showMemo: Bool {
        switch type {
        case .transfer, .deposit, .withdrawal: type.chain.isMemoSupported
        case .transferNft, .swap, .tokenApprove, .generic, .account, .stake, .perpetual, .earn: false
        }
    }
}
