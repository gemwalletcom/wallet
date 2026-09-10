// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives

struct ConfirmPaymentAssetViewModel: ItemModelProvidable {
    private let asset: Asset
    private let selectable: Bool

    init(asset: Asset, selectable: Bool) {
        self.asset = asset
        self.selectable = selectable
    }

    var itemModel: ConfirmTransferItemModel {
        .paymentAsset(
            ListItemModel(title: Localized.Transfer.payWith, subtitle: asset.symbol),
            selectable: selectable,
        )
    }
}
