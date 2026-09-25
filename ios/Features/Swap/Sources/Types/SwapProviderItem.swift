// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSwapProviderRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style

public struct SwapProviderItem: Sendable {
    public let row: GemSwapProviderRow

    public init(row: GemSwapProviderRow) {
        self.row = row
    }
}

// MARK: - List Item

public extension SwapProviderItem {
    var listItem: ListItemModel {
        ListItemModel(
            title: row.title,
            titleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            subtitle: row.amount.text(),
            subtitleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            subtitleExtra: row.fiat?.text(),
            subtitleStyleExtra: TextStyle(font: .footnote, color: Colors.gray),
            imageStyle: .asset(assetImage: AssetImage(
                placeholder: row.provider.toPrimitives().image,
                chainPlaceholder: row.isSelected ? Images.Wallets.selected : nil,
            )),
        )
    }
}

// MARK: - Identifiable

extension SwapProviderItem: Identifiable {
    public var id: String {
        row.provider.toPrimitives().rawValue
    }
}

// MARK: - Hashable

extension SwapProviderItem: Hashable {
    public static func == (lhs: SwapProviderItem, rhs: SwapProviderItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        id.hash(into: &hasher)
    }
}
