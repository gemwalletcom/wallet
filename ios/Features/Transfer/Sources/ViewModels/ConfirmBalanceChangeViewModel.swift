// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSimulationBalanceChange
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConfirmBalanceChangeViewModel {
    private let balanceChange: GemSimulationBalanceChange
    private let asset: Asset

    init(balanceChange: GemSimulationBalanceChange) {
        self.balanceChange = balanceChange
        asset = balanceChange.asset.toPrimitives()
    }

    public var listItem: ListItemModel {
        ListItemModel(
            title: assetTitle,
            titleLineLimit: 1,
            subtitle: amount.text,
            subtitleStyle: amount.style,
            imageStyle: .list(assetImage: assetImage, cornerRadiusType: .rounded),
        )
    }

    public var assetTitle: String {
        asset.name
    }

    public var assetImage: AssetImage {
        AssetImage(icon: balanceChange.icon)
    }

    public var amount: TextValue {
        TextValue(
            text: balanceChange.amount.text(),
            style: TextStyle(font: .body, color: balanceChange.amount.tone.color, fontWeight: .medium),
        )
    }
}
