// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemPerpetualDetails
import enum Gemstone.GemPerpetualDetailsAction
import struct Gemstone.PerpetualConfirmData
import Localization
import Primitives
import Style
import SwiftUI

public struct PerpetualDetailsViewModel: Sendable, Identifiable {
    public var id: String {
        details.data.baseAsset.id
    }

    private let details: GemPerpetualDetails
    private let currencyFormatter: CurrencyFormatter
    private let numericFormatter = NumericFormatter()
    private let percentFormatter = PercentFormatter.signed
    private let percentSignLessFormatter = PercentFormatter.unsigned
    private let autocloseFormatter = AutocloseFormatter(
        takeProfitLabel: Localized.Perpetual.takeProfit,
        stopLossLabel: Localized.Perpetual.stopLoss,
    )

    public init(details: GemPerpetualDetails, currencyFormatter: CurrencyFormatter = .usd) {
        self.details = details
        self.currencyFormatter = currencyFormatter
    }

    var data: PerpetualConfirmData {
        details.data
    }

    var action: GemPerpetualDetailsAction {
        details.action
    }

    public var listItemModel: ListItemModel {
        ListItemModel(
            title: Localized.Common.details,
            subtitle: listItemSubtitle,
            subtitleStyle: listItemSubtitleStyle,
        )
    }

    var positionField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Perpetual.position, style: .body),
            value: TextValue(text: positionText, style: TextStyle(font: .callout, color: directionViewModel.color)),
        )
    }

    var positionText: String {
        "\(directionViewModel.title) \(leverageText)"
    }

    var directionViewModel: PerpetualDirectionViewModel {
        PerpetualDirectionViewModel(direction: details.direction.map())
    }

    var leverageText: String {
        "\(data.leverage)x"
    }

    var slippageField: ListItemField {
        ListItemField(title: Localized.Swap.slippage, value: percentSignLessFormatter.string(data.slippage))
    }

    var marketPriceField: ListItemField {
        ListItemField(title: Localized.PriceAlerts.SetAlert.currentPrice, value: currencyFormatter.string(data.marketPrice))
    }

    var entryPriceField: ListItemField? {
        guard let price = data.entryPrice else { return nil }
        return ListItemField(title: Localized.Perpetual.entryPrice, value: currencyFormatter.string(price))
    }

    var pnlViewModel: PnLViewModel {
        PnLViewModel(
            pnl: data.pnl,
            marginAmount: data.marginAmount,
            currencyFormatter: currencyFormatter,
            percentFormatter: percentFormatter,
        )
    }

    var pnlField: ListItemField? {
        guard let text = pnlViewModel.text else { return nil }
        return ListItemField(
            title: TextValue(text: pnlViewModel.title, style: .body),
            value: TextValue(text: text, style: pnlViewModel.textStyle),
        )
    }

    var pnlText: String? {
        pnlViewModel.text
    }

    var pnlTextStyle: TextStyle {
        pnlViewModel.textStyle
    }

    var marginField: ListItemField {
        ListItemField(title: Localized.Perpetual.margin, value: currencyFormatter.string(data.marginAmount))
    }

    var sizeField: ListItemField {
        ListItemField(title: Localized.Perpetual.size, value: currencyFormatter.string(data.fiatValue))
    }

    var autocloseTitle: String {
        Localized.Perpetual.autoClose
    }

    var autocloseText: (subtitle: String, subtitleExtra: String?) {
        autocloseFormatter.format(
            takeProfit: data.takeProfit.flatMap { numericFormatter.double(from: $0) },
            stopLoss: data.stopLoss.flatMap { numericFormatter.double(from: $0) },
        )
    }

    var showAutoclose: Bool {
        data.takeProfit != nil || data.stopLoss != nil
    }
}

// MARK: - Private

extension PerpetualDetailsViewModel {
    private var listItemSubtitle: String? {
        switch action {
        case .open: String(format: "%@ %@", directionViewModel.title, leverageText)
        case .close: pnlText
        case .increase: directionViewModel.increaseTitle
        case .reduce: directionViewModel.reduceTitle
        }
    }

    private var listItemSubtitleStyle: TextStyle {
        switch action {
        case .open: TextStyle(font: .callout, color: directionViewModel.color)
        case .close: pnlTextStyle
        case .increase, .reduce: .calloutSecondary
        }
    }
}
