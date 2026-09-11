// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetRow
import Primitives
import PrimitivesComponents
import SwiftUI

struct ListAssetItemSelectionView: View {
    private let assetData: AssetData
    private let currencyCode: String
    private let row: GemAssetRow
    private let action: (ListAssetItemAction, AssetData) -> Void

    init(
        assetData: AssetData,
        currencyCode: String,
        row: GemAssetRow,
        action: @escaping (ListAssetItemAction, AssetData) -> Void,
    ) {
        self.assetData = assetData
        self.currencyCode = currencyCode
        self.row = row
        self.action = action
    }

    var body: some View {
        ListAssetItemView(
            model: ListAssetItemViewModel(
                showBalancePrivacy: .constant(false),
                assetDataModel: AssetDataViewModel(
                    assetData: assetData,
                    formatter: .short,
                    currencyCode: currencyCode,
                ),
                row: row,
                action: {
                    action($0, assetData)
                },
            ),
        )
    }
}
