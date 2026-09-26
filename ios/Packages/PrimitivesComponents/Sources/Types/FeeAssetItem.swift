// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemFeeAsset
import struct Gemstone.GemRowText
import GemstonePrimitives
import Primitives
import Style

public struct FeeAssetItem: Sendable {
    public let asset: Asset
    public let row: GemAssetItemRow
    public let isSelected: Bool

    public init(asset: Asset, row: GemAssetItemRow, isSelected: Bool) {
        self.asset = asset
        self.row = row
        self.isSelected = isSelected
    }
}

public extension FeeAssetItem {
    var listItem: ListItemModel {
        let value: (value: GemRowText, extra: GemRowText?)? = if case let .value(value, extra) = row.trailing {
            (value, extra)
        } else {
            nil
        }
        let image = AssetImage(icon: row.icon)
        return ListItemModel(
            title: row.title,
            titleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            titleExtra: row.titleExtra,
            subtitle: value?.value.text.text,
            subtitleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            subtitleExtra: value?.extra?.text.text,
            subtitleStyleExtra: TextStyle(font: .footnote, color: Colors.gray),
            imageStyle: .asset(assetImage: AssetImage(
                type: image.type,
                imageURL: image.imageURL,
                placeholder: image.placeholder,
                chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
            )),
        )
    }
}

extension FeeAssetItem: Identifiable {
    public var id: AssetId { asset.id }
}

extension FeeAssetItem: Hashable {
    public static func == (lhs: FeeAssetItem, rhs: FeeAssetItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        id.hash(into: &hasher)
    }
}

public extension FeeAssetItem {
    func selected(_ isSelected: Bool) -> FeeAssetItem {
        FeeAssetItem(asset: asset, row: row, isSelected: isSelected)
    }
}

public extension GemFeeAsset {
    var feeAssetItem: FeeAssetItem {
        FeeAssetItem(asset: asset.toPrimitives(), row: row, isSelected: false)
    }
}
