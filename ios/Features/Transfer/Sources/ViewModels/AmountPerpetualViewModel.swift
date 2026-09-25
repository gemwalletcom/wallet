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

    private let decimalSeparator = NumberInput.format(.current).decimalSeparator
    private(set) var autocloseListItem: ListItemModel?
    private var draft: GemAutocloseDraft {
        didSet { autocloseListItem = autocloseRow(draft) }
    }

    init(asset: Asset, action: GemPerpetualPositionAction, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
        (leverageSelection, leverageTextStyle) = Self.makeLeverageSelection(action: action, service: service)
        let defaults = service.perpetualAutoclose(
            action: action,
            leverage: leverageSelection?.selected.value ?? action.transferData().leverage,
            decimalSeparator: decimalSeparator,
        )
        draft = autocloseDraft(takeProfit: defaults.takeProfit, stopLoss: defaults.stopLoss)
        autocloseListItem = autocloseRow(draft)
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

    private func autocloseRow(_ draft: GemAutocloseDraft) -> ListItemModel? {
        service.perpetualAutocloseRow(draft: draft, decimalSeparator: decimalSeparator).listItemModel()
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
            draft: draft,
            decimalSeparator: decimalSeparator,
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
        let defaults = service.perpetualAutoclose(action: action, leverage: leverage, decimalSeparator: decimalSeparator)
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
        guard let leverage = service.perpetualLeverageSelection(maxLeverage: maxLeverage) else {
            return (nil, textStyle)
        }
        let options = leverage.options.map(LeverageOption.init(option:))
        let selected = LeverageOption(option: leverage.selected)
        let selection = SelectionState(
            options: options,
            selected: selected,
            isEnabled: true,
            title: Localized.Perpetual.leverage,
        )

        return (selection, textStyle)
    }
}
