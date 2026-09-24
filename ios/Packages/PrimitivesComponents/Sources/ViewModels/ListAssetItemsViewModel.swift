// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import func Gemstone.addressCopy
import func Gemstone.assetListRows
import struct Gemstone.GemAssetRowStyle
import GemstonePrimitives
import Primitives
import SwiftUI

public struct ListAssetItemsViewModel {
    private let currency: Currency
    private let rowStyle: GemAssetRowStyle

    public init(currency: Currency, rowStyle: GemAssetRowStyle) {
        self.currency = currency
        self.rowStyle = rowStyle
    }

    public func items(
        _ assetDatas: [AssetData],
        showBalancePrivacy: Binding<Bool> = .constant(false),
        action: ((ListAssetItemAction, AssetData) -> Void)? = nil,
    ) -> [ListAssetItemViewModel] {
        let models = assetDatas.map { AssetDataViewModel(assetData: $0, currency: currency) }
        let rows = assetListRows(inputs: models.map { ListAssetItemViewModel.rowInput($0, rowStyle: rowStyle) })
        return zip(models, rows).map { model, row in
            ListAssetItemViewModel(
                showBalancePrivacy: showBalancePrivacy,
                assetDataModel: model,
                rowStyle: rowStyle,
                row: row,
                action: action.map { action in { action($0, model.assetData) } },
            )
        }
    }

    public func copyMessage(chain: Chain, address: String) -> String {
        CopyTypeViewModel(content: addressCopy(chain: chain.toGem(), address: address)).message
    }
}
