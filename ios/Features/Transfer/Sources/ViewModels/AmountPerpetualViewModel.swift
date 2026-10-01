// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.autocloseDraft
import func Gemstone.autocloseOpenSession
import enum Gemstone.GemAmountRequest
import protocol Gemstone.GemAmountServiceProtocol
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseDraft
import struct Gemstone.GemAutocloseSession
import enum Gemstone.GemPerpetualPositionAction
import struct Gemstone.GemPerpetualTransferData
import func Gemstone.perpetualOpenRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
public final class AmountPerpetualViewModel {
    let asset: Asset
    let action: GemPerpetualPositionAction
    let leverageSelection: SelectionState<LeverageOption>?
    private let service: any GemAmountServiceProtocol

    private let decimalSeparator = NumberInput.format(.current).decimalSeparator
    private var draft: GemAutocloseDraft

    init(asset: Asset, action: GemPerpetualPositionAction, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
        leverageSelection = Self.makeLeverageSelection(action: action, service: service)
        let defaults = service.perpetualAutoclose(
            action: action,
            leverage: leverageSelection?.selected.value ?? action.transferData().leverage,
            decimalSeparator: decimalSeparator,
        )
        draft = autocloseDraft(takeProfit: defaults.takeProfit, stopLoss: defaults.stopLoss)
    }

    var takeProfit: String? {
        draft.takeProfit.value
    }

    var stopLoss: String? {
        draft.stopLoss.value
    }

    private var transferData: GemPerpetualTransferData {
        action.transferData()
    }

    private var leverage: UInt8 {
        leverageSelection?.selected.value ?? transferData.leverage
    }

    var request: GemAmountRequest {
        .perpetual(action: action, leverage: leverage, draft: draft, decimalSeparator: decimalSeparator)
    }

    func autocloseSession(size: Double) -> GemAutocloseSession {
        autocloseOpenSession(
            direction: transferData.direction,
            marketPrice: transferData.price,
            size: size,
            leverage: leverage,
            decimals: transferData.asset.decimals,
            provider: .hypercore,
            format: NumberInput.format(),
        )
        .onInput(tpslType: .takeProfit, text: takeProfit ?? .empty)
        .onInput(tpslType: .stopLoss, text: stopLoss ?? .empty)
    }

    func openPositionRow(size: Double) -> GemAssetItemRow {
        perpetualOpenRow(assetId: transferData.asset.id, title: transferData.asset.symbol, direction: transferData.direction, leverage: leverage, size: size)
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
    ) -> SelectionState<LeverageOption>? {
        guard case let .open(openData) = action,
              let leverage = service.perpetualLeverageSelection(maxLeverage: openData.leverage)
        else {
            return nil
        }
        return SelectionState(
            options: leverage.options.map(LeverageOption.init(option:)),
            selected: LeverageOption(option: leverage.selected),
            isEnabled: true,
            title: Localized.Perpetual.leverage,
        )
    }
}
