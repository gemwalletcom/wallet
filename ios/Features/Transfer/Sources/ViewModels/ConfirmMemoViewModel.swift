// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import Components
import Primitives
import PrimitivesComponents

struct ConfirmMemoViewModel {
    private let transfer: GemTransferData

    init(transfer: GemTransferData) {
        self.transfer = transfer
    }
}

// MARK: - ItemModelProvidable

extension ConfirmMemoViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard transfer.showsMemo() else { return .empty }
        return .memo(MemoViewModel(memo: transfer.recipient.memo).listItemModel)
    }
}
