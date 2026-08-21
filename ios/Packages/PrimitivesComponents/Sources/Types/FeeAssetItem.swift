// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import Style

struct FeeAssetItem {
    let assetData: AssetData
    let currency: Currency
    let isSelected: Bool
}

extension FeeAssetItem: SimpleListItemViewable {
    var title: String { assetData.asset.symbol }
    var titleExtra: String? { assetData.asset.name == title ? nil : assetData.asset.name }
    var subtitle: String? { model.availableBalanceTextWithSymbol }
    var subtitleExtra: String? { model.fiatBalanceText.isEmpty ? nil : model.fiatBalanceText }

    var titleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var subtitleStyleExtra: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    var assetImage: AssetImage {
        let image = AssetViewModel(asset: assetData.asset).assetImage
        return AssetImage(
            type: image.type,
            imageURL: image.imageURL,
            placeholder: image.placeholder,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
        )
    }

    private var model: AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            formatter: .short,
            currencyCode: currency.rawValue,
        )
    }
}

extension FeeAssetItem: Identifiable {
    var id: AssetId { assetData.asset.id }
}

extension FeeAssetItem: Hashable {
    static func == (lhs: FeeAssetItem, rhs: FeeAssetItem) -> Bool {
        lhs.id == rhs.id
    }

    func hash(into hasher: inout Hasher) {
        id.hash(into: &hasher)
    }
}
