// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPerpetual
import enum Gemstone.GemCurrencyStyle
import Components
import Formatters
import Foundation
import GemstonePrimitives
import struct Gemstone.GemPerpetualPositionRow
import enum Gemstone.GemPerpetualPositionDetailRow
import func Gemstone.perpetualPositionRow
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PerpetualPositionViewModel {
    public let data: PerpetualPositionData
    private let currencyFormatter: CurrencyFormatter
    private let percentFormatter = PercentFormatter.signed
    private let autocloseFormatter: AutocloseFormatter
    private let row: GemPerpetualPositionRow
    private let perpetual = GemPerpetual(provider: .hypercore)

    public init(
        _ data: PerpetualPositionData,
        currencyStyle: GemCurrencyStyle = .currency,
    ) {
        self.data = data
        row = perpetualPositionRow(perpetual: data.perpetual.toGem(), asset: data.asset.toGem(), position: data.position.toGem())
        currencyFormatter = CurrencyFormatter(type: currencyStyle, currencyCode: Currency.usd.rawValue)
        autocloseFormatter = AutocloseFormatter(
            currencyFormatter: currencyFormatter,
            takeProfitLabel: Localized.Perpetual.takeProfit,
            stopLossLabel: Localized.Perpetual.stopLoss,
        )
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

    public func detailField(for detailRow: GemPerpetualPositionDetailRow) -> ListItemField {
        switch detailRow {
        case .pnl: ListItemField(title: TextValue(text: detailRow.title, style: .body), value: TextValue(text: pnlViewModel.text ?? "", style: pnlViewModel.textStyle))
        case .autoclose: ListItemField(title: detailRow.title, value: autocloseText.subtitle)
        case .size: ListItemField(title: detailRow.title, value: currencyFormatter.string(data.position.sizeValue))
        case .entryPrice: ListItemField(title: detailRow.title, value: currencyFormatter.string(data.position.entryPrice))
        case .liquidationPrice:
            ListItemField(
                title: TextValue(text: detailRow.title, style: .body),
                value: TextValue(text: row.liquidationPrice?.text() ?? Placeholder.empty, style: liquidationPriceTextStyle),
            )
        case .margin:
            ListItemField(
                title: detailRow.title,
                value: perpetual.marginText(
                    formattedAmount: currencyFormatter.string(data.position.marginAmount),
                    marginTypeName: data.position.marginType.title,
                ),
            )
        case .fundingPayments:
            ListItemField(
                title: TextValue(text: detailRow.title, style: .body),
                value: TextValue(text: fundingPaymentsModel.text ?? Placeholder.empty, style: fundingPaymentsModel.textStyle),
            )
        }
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

    var autocloseText: (subtitle: String, subtitleExtra: String?) {
        autocloseFormatter.format(
            takeProfit: data.position.takeProfit?.price,
            stopLoss: data.position.stopLoss?.price,
        )
    }

}

// MARK: - Private

extension PerpetualPositionViewModel {
    var fundingPaymentsModel: PriceChangeViewModel {
        PriceChangeViewModel(value: data.position.funding.map { Double($0) }, currencyFormatter: currencyFormatter)
    }

    var liquidationPriceTextStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.secondaryText)
    }
}

extension PerpetualPositionViewModel: Identifiable {
    public var id: String {
        data.position.id
    }
}
