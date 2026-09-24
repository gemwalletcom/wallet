// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import func Gemstone.autocloseDraft
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountType
import struct Gemstone.GemAutocloseDraft
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
    private let service: any GemAmountServiceProtocol

    private var draft: GemAutocloseDraft

    init(asset: Asset, action: GemPerpetualPositionAction, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
        (leverageSelection, leverageTextStyle) = Self.makeLeverageSelection(action: action, service: service)
        let defaults = Self.makeDefaultAutoclose(action: action, leverage: leverageSelection?.selected.value ?? action.transferData().leverage, service: service)
        draft = autocloseDraft(takeProfit: defaults.takeProfit, stopLoss: defaults.stopLoss)
    }

    var takeProfit: String? {
        draft.takeProfit.value
    }

    var stopLoss: String? {
        draft.stopLoss.value
    }

    var leverageListItem: ListItemModel? {
        leverageSelection.map { ListItemModel(title: $0.title, subtitle: $0.selected.displayText, subtitleStyle: leverageTextStyle) }
    }

    var autocloseListItem: ListItemModel? {
        service.perpetualAutocloseRow(
            takeProfit: takeProfit.flatMap { NumberInput.double($0) },
            stopLoss: stopLoss.flatMap { NumberInput.double($0) },
        ).listItemModel()
    }

    private var transferData: GemPerpetualTransferData {
        action.transferData()
    }

    private var leverage: UInt8 {
        leverageSelection?.selected.value ?? transferData.leverage
    }

    var isAutocloseEnabled: Bool {
        action.showsAutoclose()
    }

    private var direction: PerpetualDirection {
        transferData.direction.toPrimitives()
    }

    var title: String {
        gemAmountType.title().title
    }

    var gemAmountType: GemAmountType {
        service.perpetualAmountType(action: action, leverage: leverage)
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) -> GemTransferData {
        service.perpetualTransferData(
            action: action,
            value: value,
            useMaxAmount: useMaxAmount,
            leverage: leverage,
            takeProfit: takeProfit.flatMap { NumberInput.double($0) },
            stopLoss: stopLoss.flatMap { NumberInput.double($0) },
        )
    }

    func makeAutocloseData(size: Double) -> AutocloseOpenData {
        AutocloseOpenData(
            assetId: transferData.asset.toPrimitives().id,
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
        let defaults = Self.makeDefaultAutoclose(action: action, leverage: leverage, service: service)
        draft = draft.onDefaults(takeProfit: defaults.takeProfit, stopLoss: defaults.stopLoss)
    }

    func updateAutoclose(takeProfit: String?, stopLoss: String?) {
        draft = draft
            .onEdited(tpslType: .takeProfit, value: takeProfit)
            .onEdited(tpslType: .stopLoss, value: stopLoss)
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
            color: openData.direction.toPrimitives().color,
        )
        let options = service.perpetualLeverageOptions(maxLeverage: maxLeverage).map(LeverageOption.init(option:))
        guard let selected = LeverageOption.selected(service.perpetualLeverage(maxLeverage: maxLeverage), in: options) else {
            return (nil, textStyle)
        }
        let selection = SelectionState(
            options: options,
            selected: selected,
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
