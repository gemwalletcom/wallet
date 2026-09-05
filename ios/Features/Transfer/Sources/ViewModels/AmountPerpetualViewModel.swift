// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import enum Gemstone.GemAmountPerpetualPosition
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountType
import struct Gemstone.GemPaymentRecipient
import enum Gemstone.GemPerpetualPositionAction
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
    let action: GemPerpetualPositionAction
    let leverageSelection: SelectionState<LeverageOption>?
    let leverageTextStyle: TextStyle
    let currencyFormatter: CurrencyFormatter
    private let service: any GemAmountServiceProtocol
    private let numericFormatter = NumericFormatter()

    var takeProfit: String?
    var stopLoss: String?

    private var isAutocloseEdited = false

    init(asset: Asset, action: GemPerpetualPositionAction, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: service.getCurrency())
        (leverageSelection, leverageTextStyle) = Self.makeLeverageSelection(action: action, service: service)
        (takeProfit, stopLoss) = Self.makeDefaultAutoclose(action: action, leverage: leverageSelection?.selected.value ?? action.transferData().leverage, service: service)
    }

    var leverageTitle: String {
        Localized.Perpetual.leverage
    }

    var autocloseTitle: String {
        Localized.Perpetual.autoClose
    }

    private var transferData: GemPerpetualTransferData {
        action.transferData()
    }

    private var leverage: UInt8 {
        leverageSelection?.selected.value ?? transferData.leverage
    }

    var isAutocloseEnabled: Bool {
        switch action {
        case .open: true
        case .increase, .reduce: false
        }
    }

    private var direction: PerpetualDirection {
        transferData.direction.map()
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
        switch action {
        case .open: PerpetualDirectionViewModel(direction: direction).title
        case .increase: PerpetualDirectionViewModel(direction: direction).increaseTitle
        case .reduce: PerpetualDirectionViewModel(direction: direction).reduceTitle
        }
    }

    var amountType: AmountType {
        .perpetual(action)
    }

    var gemAmountType: GemAmountType {
        let position: GemAmountPerpetualPosition = switch action {
        case .open: .open
        case .increase: .increase
        case let .reduce(_, available): .reduce(available: available)
        }
        return .perpetual(position: position, price: transferData.price, leverage: leverage, sizeDecimals: transferData.asset.decimals)
    }

    func recipientData() -> GemPaymentRecipient {
        GemPaymentRecipient(recipient: action.recipient())
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) -> GemTransferData {
        service.perpetualTransferData(
            action: action,
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
        (takeProfit, stopLoss) = Self.makeDefaultAutoclose(action: action, leverage: leverage, service: service)
    }

    func updateAutoclose(takeProfit: String?, stopLoss: String?) {
        isAutocloseEdited = true
        self.takeProfit = takeProfit
        self.stopLoss = stopLoss
    }

    private static func makeLeverageSelection(
        action: GemPerpetualPositionAction,
        service: any GemAmountServiceProtocol,
    ) -> (SelectionState<LeverageOption>?, TextStyle) {
        guard case let .open(openData) = action else {
            return (nil, .callout)
        }

        let maxLeverage = openData.leverage
        let textStyle = TextStyle(
            font: .callout,
            color: PerpetualDirectionViewModel(direction: openData.direction.map()).color,
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
        action: GemPerpetualPositionAction,
        leverage: UInt8,
        service: any GemAmountServiceProtocol,
    ) -> (takeProfit: String?, stopLoss: String?) {
        guard case .open = action else {
            return (nil, nil)
        }
        let transferData = action.transferData()
        let autoclose = service.perpetualAutoclose(price: transferData.price, direction: transferData.direction, leverage: leverage)
        let formatter = PerpetualFormatter(provider: .hypercore)
        return (
            autoclose.takeProfit.map { formatter.formatInputPrice($0, decimals: transferData.asset.decimals) },
            autoclose.stopLoss.map { formatter.formatInputPrice($0, decimals: transferData.asset.decimals) },
        )
    }
}
