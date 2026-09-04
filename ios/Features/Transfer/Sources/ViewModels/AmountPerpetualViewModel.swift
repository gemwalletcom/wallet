// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import enum Gemstone.GemAmountPerpetualPosition
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountType
import struct Gemstone.GemPerpetualTransferData
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Localization
import Perpetuals
import Primitives
import PrimitivesComponents
import Style

@Observable
public final class AmountPerpetualViewModel: AmountDataProvidable {
    let asset: Asset
    let data: PerpetualRecipientData
    let leverageSelection: SelectionState<LeverageOption>?
    let leverageTextStyle: TextStyle
    let currencyFormatter: CurrencyFormatter
    private let service: any GemAmountServiceProtocol
    private let numericFormatter = NumericFormatter()

    var takeProfit: String?
    var stopLoss: String?

    private var isAutocloseEdited = false

    init(asset: Asset, data: PerpetualRecipientData, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.data = data
        self.service = service
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: service.getCurrency())
        (leverageSelection, leverageTextStyle) = Self.makeLeverageSelection(data: data, service: service)
        (takeProfit, stopLoss) = Self.makeDefaultAutoclose(data: data, leverage: leverageSelection?.selected.value ?? data.positionAction.transferData().leverage, service: service)
    }

    var leverageTitle: String {
        Localized.Perpetual.leverage
    }

    var autocloseTitle: String {
        Localized.Perpetual.autoClose
    }

    private var transferData: GemPerpetualTransferData {
        data.positionAction.transferData()
    }

    private var leverage: UInt8 {
        leverageSelection?.selected.value ?? transferData.leverage
    }

    var isAutocloseEnabled: Bool {
        switch data.positionAction {
        case .open: true
        case .increase, .reduce: false
        }
    }

    private var direction: PerpetualDirection {
        PerpetualDirection(core: transferData.direction)
    }

    var autocloseText: (subtitle: String, subtitleExtra: String?) {
        AutocloseFormatter(
            takeProfitLabel: Localized.Perpetual.takeProfit,
            stopLossLabel: Localized.Perpetual.stopLoss,
        ).format(
            takeProfit: takeProfit.flatMap { numericFormatter.double(from: $0) },
            stopLoss: stopLoss.flatMap { numericFormatter.double(from: $0) },
        )
    }

    var title: String {
        switch data.positionAction {
        case .open: PerpetualDirectionViewModel(direction: direction).title
        case .increase: PerpetualDirectionViewModel(direction: direction).increaseTitle
        case .reduce: PerpetualDirectionViewModel(direction: direction).reduceTitle
        }
    }

    var amountType: AmountType {
        .perpetual(data)
    }

    var gemAmountType: GemAmountType {
        let position: GemAmountPerpetualPosition = switch data.positionAction {
        case .open: .open
        case .increase: .increase
        case let .reduce(_, available): .reduce(available: available)
        }
        return .perpetual(position: position, price: transferData.price, leverage: leverage, sizeDecimals: transferData.asset.decimals)
    }

    func recipientData() -> RecipientData {
        data.recipient
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) -> GemTransferData {
        service.perpetualTransferData(
            action: data.positionAction,
            value: value,
            useMaxAmount: useMaxAmount,
            leverage: leverage,
            takeProfit: takeProfit.flatMap { numericFormatter.double(from: $0) },
            stopLoss: stopLoss.flatMap { numericFormatter.double(from: $0) },
        )
    }

    func makeAutocloseData(size: Double) -> AutocloseOpenData {
        AutocloseOpenData(
            assetId: transferData.asset.map().id,
            symbol: transferData.asset.symbol,
            direction: direction,
            marketPrice: transferData.price,
            leverage: leverageSelection?.selected.value ?? 1,
            size: size,
            assetDecimals: transferData.asset.decimals,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
    }

    func onChangeLeverage() {
        guard !isAutocloseEdited else { return }
        (takeProfit, stopLoss) = Self.makeDefaultAutoclose(data: data, leverage: leverage, service: service)
    }

    func updateAutoclose(takeProfit: String?, stopLoss: String?) {
        isAutocloseEdited = true
        self.takeProfit = takeProfit
        self.stopLoss = stopLoss
    }

    private static func makeLeverageSelection(
        data: PerpetualRecipientData,
        service: any GemAmountServiceProtocol,
    ) -> (SelectionState<LeverageOption>?, TextStyle) {
        guard case let .open(openData) = data.positionAction else {
            return (nil, .callout)
        }

        let maxLeverage = openData.leverage
        let textStyle = TextStyle(
            font: .callout,
            color: PerpetualDirectionViewModel(direction: PerpetualDirection(core: openData.direction)).color,
        )
        let selection = SelectionState(
            options: LeverageOption.options(maxLeverage: maxLeverage),
            selected: LeverageOption(value: service.perpetualLeverage(maxLeverage: maxLeverage)),
            isEnabled: true,
            title: Localized.Perpetual.leverage,
        )

        return (selection, textStyle)
    }

    private static func makeDefaultAutoclose(
        data: PerpetualRecipientData,
        leverage: UInt8,
        service: any GemAmountServiceProtocol,
    ) -> (takeProfit: String?, stopLoss: String?) {
        guard case .open = data.positionAction else {
            return (nil, nil)
        }
        let transferData = data.positionAction.transferData()
        let autoclose = service.perpetualAutoclose(price: transferData.price, direction: transferData.direction, leverage: leverage)
        let formatter = PerpetualFormatter(provider: .hypercore)
        return (
            autoclose.takeProfit.map { formatter.formatInputPrice($0, decimals: transferData.asset.decimals) },
            autoclose.stopLoss.map { formatter.formatInputPrice($0, decimals: transferData.asset.decimals) },
        )
    }
}
