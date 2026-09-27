// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.autocloseOpenSession
import func Gemstone.autocloseSession
import struct Gemstone.GemAssetItemRow
import enum Gemstone.GemAutocloseConfirmPolicy
import struct Gemstone.GemAutocloseSession
import struct Gemstone.GemAutocloseViewState
import func Gemstone.perpetualOpenRow
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

    var focusField: AutocloseScene.Field?
    var isPresentingAlertMessage: AlertMessage?
    private var session: GemAutocloseSession

    public init(type: AutocloseType) {
        self.type = type
        session = Self.session(for: type)
    }

    private static func session(for type: AutocloseType) -> GemAutocloseSession {
        switch type {
        case let .modify(position, _):
            autocloseSession(
                perpetual: position.perpetual.toGem(),
                asset: position.asset.toGem(),
                position: position.position.toGem(),
                format: NumberInput.format(),
            )
        case let .open(data, _):
            autocloseOpenSession(
                direction: data.direction.toGem(),
                marketPrice: data.marketPrice,
                size: data.size,
                leverage: data.leverage,
                decimals: data.assetDecimals,
                provider: .hypercore,
                format: NumberInput.format(),
            )
            .onInput(tpslType: .takeProfit, text: data.takeProfit ?? .empty)
            .onInput(tpslType: .stopLoss, text: data.stopLoss ?? .empty)
        }
    }

    public var title: String {
        Localized.Perpetual.autoClose
    }

    var viewState: GemAutocloseViewState {
        session.viewState()
    }

    var takeProfitText: String {
        get { viewState.takeProfit.text }
        set { session = session.onInput(tpslType: .takeProfit, text: newValue) }
    }

    var stopLossText: String {
        get { viewState.stopLoss.text }
        set { session = session.onInput(tpslType: .stopLoss, text: newValue) }
    }

    func percentSuggestions(_ viewState: GemAutocloseViewState) -> [PercentageSuggestion] {
        viewState.takeProfit.suggestions.map { PercentageSuggestion(number: $0) }
    }

    func positionRow(_ viewState: GemAutocloseViewState) -> GemAssetItemRow? {
        switch type {
        case .modify: viewState.positionRow?.row
        case let .open(data, _): perpetualOpenRow(assetId: data.assetId.identifier, title: data.symbol, direction: data.direction.toGem(), leverage: data.leverage, size: data.size)
        }
    }

    func confirmButtonType(_ viewState: GemAutocloseViewState) -> ButtonType {
        .primary(viewState.confirmEnabled ? .normal : .disabled)
    }
}

// MARK: - Actions

public extension AutocloseSceneViewModel {
    func isEditing(field: AutocloseScene.Field?) -> Bool {
        switch field {
        case .takeProfit: takeProfitText.isEmpty
        case .stopLoss: stopLossText.isEmpty
        case nil: false
        }
    }

    func onChangeFocusField(_ _: AutocloseScene.Field?, _ newField: AutocloseScene.Field?) {
        focusField = newField
    }

    func onSelectConfirm() {
        let attempted = session.onSubmitAttempt()
        session = attempted
        let state = attempted.viewState()
        guard state.confirmEnabled else { return }

        switch type {
        case let .modify(position, onTransferAction):
            do {
                try onTransferAction?(attempted.modify.transfer(provider: position.perpetual.provider.toGem(), asset: position.asset.toGem()))
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }

        case let .open(_, onComplete):
            onComplete(state.takeProfit.text, state.stopLoss.text)
        }
    }

    func onSelectPercent(_ percent: Int) {
        let type: TpslType
        switch focusField {
        case .takeProfit: type = .takeProfit
        case .stopLoss: type = .stopLoss
        case nil: return
        }
        session = session.onPercentSelected(tpslType: type.toGem(), percent: Int32(percent))
    }
}
