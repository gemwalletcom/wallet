// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemCurrencyStyle
import class Gemstone.GemPerpetual
import struct Gemstone.GemPerpetualPositionRow
import func Gemstone.perpetualPositionRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import SwiftUI

public struct PerpetualPositionViewModel {
    public let data: PerpetualPositionData
    private let currencyFormatter: CurrencyFormatter
    private let percentFormatter = PercentFormatter.signed
    private let row: GemPerpetualPositionRow
    private let perpetual = GemPerpetual(provider: .hypercore)

    public init(
        _ data: PerpetualPositionData,
        currencyStyle: GemCurrencyStyle = .currency,
    ) {
        self.data = data
        row = perpetualPositionRow(perpetual: data.perpetual.toGem(), asset: data.asset.toGem(), position: data.position.toGem())
        currencyFormatter = CurrencyFormatter(type: currencyStyle, currencyCode: Currency.usd.rawValue)
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: data.perpetual.assetId).assetImage
    }

    public var symbolText: String {
        row.title
    }

    public var leverageText: String {
        row.leverage
    }

    public var directionText: String {
        PerpetualDirectionViewModel(direction: data.position.direction).title
    }

    public var positionTypeText: String {
        perpetual.positionText(directionName: directionText, formattedLeverage: leverageText)
    }

    public var positionTypeColor: Color {
        PerpetualDirectionViewModel(direction: data.position.direction).color
    }

    public var pnlViewModel: PnLViewModel {
        PnLViewModel(
            pnl: data.position.pnl,
            marginAmount: data.position.marginAmount,
            currencyFormatter: currencyFormatter,
            percentFormatter: percentFormatter,
        )
    }

    public var pnlColor: Color {
        pnlViewModel.color
    }

    public var pnlWithPercentText: String {
        pnlViewModel.text ?? ""
    }

    public var marginAmountText: String {
        currencyFormatter.string(data.position.marginAmount)
    }
}

extension PerpetualPositionViewModel: Identifiable {
    public var id: String {
        data.position.id
    }
}
