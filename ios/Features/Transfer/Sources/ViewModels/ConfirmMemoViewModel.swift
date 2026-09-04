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
        guard type.showsMemo() else { return .empty }
        return .memo(MemoViewModel(memo: recipient.memo).listItemModel)
    }
}
