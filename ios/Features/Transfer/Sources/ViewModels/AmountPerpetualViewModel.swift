// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.autocloseDraft
import enum Gemstone.GemAmountRequest
import protocol Gemstone.GemAmountServiceProtocol
import struct Gemstone.GemAutocloseDraft
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
