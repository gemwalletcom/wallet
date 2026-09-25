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

extension FeeAssetItem: SimpleListItemViewable {
    public var title: String { row.title }
    public var titleExtra: String? { row.titleExtra }
    public var subtitle: String? { trailingValue?.value.text.text }
    public var subtitleExtra: String? { trailingValue?.extra?.text.text }

    public var titleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    public var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    public var subtitleStyleExtra: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    public var assetImage: AssetImage {
        let image = AssetImage(icon: row.icon)
        return AssetImage(
            type: image.type,
            imageURL: image.imageURL,
            placeholder: image.placeholder,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
        )
    }

    private var trailingValue: (value: GemRowText, extra: GemRowText?)? {
        guard case let .value(value, extra) = row.trailing else { return nil }
        return (value, extra)
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
