// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemProviderKind
import struct Gemstone.GemProviderRow
import GemstonePrimitives
import Style
import SwiftUI

extension GemProviderRow: @retroactive Identifiable {
    public var id: GemProviderKind {
        kind
    }
}

public extension GemProviderRow {
    var listItem: ListItemModel {
        ListItemModel(
            title: name,
            titleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            subtitle: amount.text(),
            subtitleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            subtitleExtra: fiat?.text(),
            subtitleStyleExtra: TextStyle(font: .footnote, color: Colors.gray),
            imageStyle: .asset(assetImage: AssetImage(
                placeholder: kind.image,
                chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
            )),
        )
    }
}

private extension GemProviderKind {
    var image: Image {
        switch self {
        case let .swap(provider): provider.toPrimitives().image
        case let .fiat(provider): provider.toPrimitives().image
        }
    }
}
