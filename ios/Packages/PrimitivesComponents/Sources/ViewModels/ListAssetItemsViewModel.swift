// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemAssetRow
import Primitives
import SwiftUI

public struct ListAssetItemsViewModel {
    private let currency: Currency
    private let row: GemAssetRow

    public init(currency: Currency, row: GemAssetRow) {
        self.currency = currency
        self.row = row
    }

    public func item(
        _ assetData: AssetData,
        showBalancePrivacy: Binding<Bool> = .constant(false),
        action: ((ListAssetItemAction) -> Void)? = nil,
    ) -> ListAssetItemViewModel {
        ListAssetItemViewModel(
            showBalancePrivacy: showBalancePrivacy,
            assetDataModel: AssetDataViewModel(assetData: assetData, formatter: .short, currency: currency),
            row: row,
            action: action,
        )
    }
}
