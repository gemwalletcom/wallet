// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.autocloseOpenSession
import func Gemstone.autocloseSession
import enum Gemstone.GemAutocloseConfirmPolicy
import struct Gemstone.GemAutocloseSession
import struct Gemstone.GemAutocloseViewState
import enum Gemstone.GemListRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AutocloseSceneViewModel {
    private let type: AutocloseType
    private let decimalSeparator = NumberInput.format(.current).decimalSeparator

    var input: AutocloseInput
    private var session: GemAutocloseSession

    public init(type: AutocloseType) {
        let session = Self.session(for: type)
        let separator = NumberInput.format(.current).decimalSeparator

        self.type = type
        self.session = session
        input = AutocloseInput(
            takeProfitText: session.inputText(tpslType: .takeProfit, decimalSeparator: separator),
            stopLossText: session.inputText(tpslType: .stopLoss, decimalSeparator: separator),
        )
    }

    private static func session(for type: AutocloseType) -> GemAutocloseSession {
        switch type {
        case let .modify(position, _):
            autocloseSession(
                perpetual: position.perpetual.toGem(),
                asset: position.asset.toGem(),
                position: position.position.toGem(),
            )
        case let .open(data, _):
            autocloseOpenSession(
                direction: data.direction.toGem(),
                marketPrice: data.marketPrice,
                size: data.size,
                leverage: data.leverage,
                decimals: data.assetDecimals,
                provider: .hypercore,
            )
            .onPrice(tpslType: .takeProfit, price: data.takeProfit.flatMap { NumberInput.double($0) })
            .onPrice(tpslType: .stopLoss, price: data.stopLoss.flatMap { NumberInput.double($0) })
        }
    }

    public var title: String {
        Localized.Perpetual.autoClose
    }

    public var priceRows: [GemListRow] {
        viewState.priceRows
    }

    public var takeProfitModel: AutocloseViewModel {
        AutocloseViewModel(state: viewState.takeProfit)
    }

    public var stopLossModel: AutocloseViewModel {
        AutocloseViewModel(state: viewState.stopLoss)
    }

    public var positionItemViewModel: (any ListAssetItemViewable)? {
        switch type {
        case .modify: viewState.positionRow.map { PerpetualPositionItemViewModel(row: $0) }
        case let .open(data, _): OpenPositionItemViewModel(data: data)
        }
    }

    private var viewState: GemAutocloseViewState {
        session.viewState()
    }

    public var confirmButtonType: ButtonType {
        .primary(viewState.confirmEnabled ? .normal : .disabled)
    }
}

// MARK: - Actions

public extension AutocloseSceneViewModel {
    func isEditing(field: AutocloseScene.Field?) -> Bool {
        guard let field else { return false }
        return input.text(for: field).isEmpty
    }

    func onChangeFocusField(_ _: AutocloseScene.Field?, _ newField: AutocloseScene.Field?) {
        input.focusField = newField
    }

    func onChangePrice() {
        session = session
            .onPrice(tpslType: .takeProfit, price: takeProfitPrice)
            .onPrice(tpslType: .stopLoss, price: stopLossPrice)
    }

    func onSelectConfirm() {
        onChangePrice()

        let attempted = session.onSubmitAttempt()
        session = attempted
        let state = attempted.viewState()
        input.update(state: state)
        guard state.confirmEnabled else { return }

        switch type {
        case let .modify(position, onTransferAction):
            guard let transfer = try? attempted.modify.transfer(provider: position.perpetual.provider.toGem(), asset: position.asset.toGem()) else { return }
            onTransferAction?(transfer)

        case let .open(_, onComplete):
            onComplete(input.selection)
        }
    }

    func onSelectPercent(_ percent: Int) {
        guard let type = input.focusedType, let focused = input.focused else { return }
        session = session.onPercentSelected(tpslType: type.toGem(), percent: Int32(percent))
        focused.text = session.inputText(tpslType: type.toGem(), decimalSeparator: decimalSeparator) ?? .empty
    }
}

// MARK: - Private

extension AutocloseSceneViewModel {
    private var takeProfitPrice: Double? {
        NumberInput.double(input.takeProfit.text)
    }

    private var stopLossPrice: Double? {
        NumberInput.double(input.stopLoss.text)
    }
}
