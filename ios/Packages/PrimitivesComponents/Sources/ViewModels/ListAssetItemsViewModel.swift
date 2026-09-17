// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemAssetRowStyle
import Primitives
import SwiftUI

public struct ListAssetItemsViewModel {
    private let currency: Currency
    private let rowStyle: GemAssetRowStyle

    public init(currency: Currency, rowStyle: GemAssetRowStyle) {
        self.currency = currency
        self.rowStyle = rowStyle
    }

    public func item(
        _ assetData: AssetData,
        showBalancePrivacy: Binding<Bool> = .constant(false),
        action: ((ListAssetItemAction) -> Void)? = nil,
    ) -> ListAssetItemViewModel {
        ListAssetItemViewModel(
            showBalancePrivacy: showBalancePrivacy,
            assetDataModel: AssetDataViewModel(assetData: assetData, formatter: .short, currency: currency),
            rowStyle: rowStyle,
            action: action,
        )
    }
}
