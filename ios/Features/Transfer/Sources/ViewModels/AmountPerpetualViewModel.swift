// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.autocloseDraft
import enum Gemstone.GemAmountRequest
import protocol Gemstone.GemAmountServiceProtocol
import struct Gemstone.GemAutocloseDraft
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemPerpetualPositionAction
import struct Gemstone.GemPerpetualTransferData
import GemstonePrimitives
import Localization
import Perpetuals
import Primitives
import PrimitivesComponents
import Style

@Observable
public final class AmountPerpetualViewModel {
    let asset: Asset
    let action: GemPerpetualPositionAction
    let leverageSelection: SelectionState<LeverageOption>?
    let leverageTextStyle: TextStyle
    private let service: any GemAmountServiceProtocol

    private let decimalSeparator = NumberInput.format(.current).decimalSeparator
    var onInfo: ((GemInfoTopic) -> Void)?
    private(set) var autocloseRow: GemListRow
    private var draft: GemAutocloseDraft {
        didSet { autocloseRow = Self.autocloseRow(draft, service: service, decimalSeparator: decimalSeparator) }
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
        let draft = autocloseDraft(takeProfit: defaults.takeProfit, stopLoss: defaults.stopLoss)
        self.draft = draft
        autocloseRow = Self.autocloseRow(draft, service: service, decimalSeparator: decimalSeparator)
    }

    var autocloseListItem: ListItemModel? {
        autocloseRow.listItemModel(onInfo: onInfo)
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

    private static func autocloseRow(_ draft: GemAutocloseDraft, service: any GemAmountServiceProtocol, decimalSeparator: String) -> GemListRow {
        service.perpetualAutocloseRow(draft: draft, decimalSeparator: decimalSeparator)
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

    var request: GemAmountRequest {
        .perpetual(action: action, leverage: leverage, draft: draft, decimalSeparator: decimalSeparator)
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
