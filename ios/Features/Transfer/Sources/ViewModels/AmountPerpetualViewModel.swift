// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import enum Gemstone.GemAmountType
import enum Gemstone.GemAmountPerpetualPosition
import GemstonePrimitives
import Localization
import Perpetuals
import GemstoneServices
import Primitives
import PrimitivesComponents
import Style
import class Gemstone.GemAutocloseEstimator

@Observable
public final class AmountPerpetualViewModel: AmountDataProvidable {
    let asset: Asset
    let data: PerpetualRecipientData
    let leverageSelection: SelectionState<LeverageOption>?
    let leverageTextStyle: TextStyle
    let currencyFormatter: CurrencyFormatter
    private let numericFormatter = NumericFormatter()

    var takeProfit: String?
    var stopLoss: String?

    private let takeProfitPercent: UInt8
    private let stopLossPercent: UInt8
    private var isAutocloseEdited = false

    init(asset: Asset, data: PerpetualRecipientData, preferencesService: any GemPreferencesServiceProtocol) {
        self.asset = asset
        self.data = data
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: preferencesService.currencyCode)
        takeProfitPercent = preferencesService.getPerpetualTakeProfitPercent()
        stopLossPercent = preferencesService.getPerpetualStopLossPercent()
        (leverageSelection, leverageTextStyle) = Self.makeLeverageSelection(data: data, leverage: preferencesService.getPerpetualLeverage())
        (takeProfit, stopLoss) = Self.makeDefaultAutoclose(
            data: data,
            leverage: leverageSelection?.selected.value ?? data.positionAction.transferData.leverage,
            takeProfitPercent: takeProfitPercent,
            stopLossPercent: stopLossPercent,
        )
    }

    var leverageTitle: String {
        Localized.Perpetual.leverage
    }

    var autocloseTitle: String {
        Localized.Perpetual.autoClose
    }

    private var transferData: PerpetualTransferData {
        data.positionAction.transferData
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
        case .open:
            PerpetualDirectionViewModel(direction: transferData.direction).title
        case .increase:
            PerpetualDirectionViewModel(direction: transferData.direction).increaseTitle
        case let .reduce(_, _, direction):
            PerpetualDirectionViewModel(direction: direction).reduceTitle
        }
    }

    var amountType: AmountType {
        .perpetual(data)
    }

    var gemAmountType: GemAmountType {
        let position: GemAmountPerpetualPosition = switch data.positionAction {
        case .open: .open
        case .increase: .increase
        case let .reduce(_, available, _): .reduce(available: available.description)
        }
        return .perpetual(position: position, price: transferData.price, leverage: leverage, sizeDecimals: transferData.asset.decimals)
    }

    func recipientData() -> RecipientData {
        data.recipient
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) throws -> TransferData {
        let formatter = PerpetualFormatter(provider: .hypercore)

        let perpetualType = try data.positionAction.order(
            usdcAmount: value,
            usdcDecimals: asset.decimals.asInt,
            leverage: leverage,
            takeProfit: takeProfit
                .flatMap { numericFormatter.double(from: $0) }
                .map { formatter.formatPrice($0, decimals: transferData.asset.decimals) },
            stopLoss: stopLoss
                .flatMap { numericFormatter.double(from: $0) }
                .map { formatter.formatPrice($0, decimals: transferData.asset.decimals) },
        )

        return TransferData(
            type: .perpetual(transferData.asset, perpetualType),
            recipient: data.recipient.recipient,
            value: value,
            useMaxAmount: useMaxAmount,
        )
    }

    func makeAutocloseData(size: Double) -> AutocloseOpenData {
        AutocloseOpenData(
            assetId: transferData.asset.id,
            symbol: transferData.asset.symbol,
            direction: transferData.direction,
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
        (takeProfit, stopLoss) = Self.makeDefaultAutoclose(
            data: data,
            leverage: leverage,
            takeProfitPercent: takeProfitPercent,
            stopLossPercent: stopLossPercent,
        )
    }

    func updateAutoclose(takeProfit: String?, stopLoss: String?) {
        isAutocloseEdited = true
        self.takeProfit = takeProfit
        self.stopLoss = stopLoss
    }

    private static func makeLeverageSelection(
        data: PerpetualRecipientData,
        leverage: UInt8,
    ) -> (SelectionState<LeverageOption>?, TextStyle) {
        guard case let .open(openData) = data.positionAction else {
            return (nil, .callout)
        }

        let transferData = data.positionAction.transferData
        let options = LeverageOption.options(maxLeverage: transferData.leverage)
        let selected = LeverageOption.option(desiredValue: leverage, from: options)
        let textStyle = TextStyle(
            font: .callout,
            color: PerpetualDirectionViewModel(direction: openData.direction).color,
        )

        let selection = SelectionState(
            options: options,
            selected: selected,
            isEnabled: true,
            title: Localized.Perpetual.leverage,
        )

        return (selection, textStyle)
    }

    private static func makeDefaultAutoclose(
        data: PerpetualRecipientData,
        leverage: UInt8,
        takeProfitPercent: UInt8,
        stopLossPercent: UInt8,
    ) -> (takeProfit: String?, stopLoss: String?) {
        guard case .open = data.positionAction else {
            return (nil, nil)
        }
        guard takeProfitPercent > 0 || stopLossPercent > 0 else {
            return (nil, nil)
        }

        let transferData = data.positionAction.transferData
        let estimator = GemAutocloseEstimator(
            entryPrice: transferData.price,
            positionSize: 0,
            direction: transferData.direction.json(),
            leverage: leverage,
        )
        let formatter = PerpetualFormatter(provider: .hypercore)
        func price(_ percent: UInt8, _ type: TpslType) -> String? {
            guard percent > 0 else { return nil }
            return formatter.formatInputPrice(
                estimator.targetPriceFromRoe(roePercent: Int32(percent), triggerType: type.map()),
                decimals: transferData.asset.decimals,
            )
        }
        return (price(takeProfitPercent, .takeProfit), price(stopLossPercent, .stopLoss))
    }
}
