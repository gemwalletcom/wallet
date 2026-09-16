// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import Components
import GemstonePrimitives
import Localization
import Primitives

struct ConfirmPaymentAssetViewModel: ItemModelProvidable {
    private let transfer: GemTransferData

    init(transfer: GemTransferData) {
        self.transfer = transfer
    }

    var itemModel: ConfirmTransferItemModel {
        guard let invoice = transfer.invoice else {
            return .empty
        }
        return .paymentAsset(ListItemModel(title: Localized.Transfer.payWith, subtitle: transfer.asset.symbol), selectable: invoice.quotes.count > 1)
    }
}
