// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.autocloseOpenSession
import func Gemstone.autocloseSession
import enum Gemstone.GemAutocloseConfirmPolicy
import class Gemstone.GemAutocloseEstimator
import struct Gemstone.GemAutocloseSession
import struct Gemstone.GemAutocloseViewState
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AutocloseSceneViewModel {
    private let currencyFormatter: CurrencyFormatter
    private let perpetualFormatter = PerpetualFormatter(provider: .hypercore)
    private let type: AutocloseType
    private let estimator: GemAutocloseEstimator

    var input: AutocloseInput
    private var session: GemAutocloseSession

    private static func estimator(for type: AutocloseType) -> GemAutocloseEstimator {
        switch type {
        case let .modify(position, _):
            GemAutocloseEstimator(
                entryPrice: position.position.entryPrice,
                positionSize: position.position.size,
                direction: position.position.direction.toGem(),
                leverage: position.position.leverage,
            )
        case let .open(data, _):
            GemAutocloseEstimator.forOpen(
                marketPrice: data.marketPrice,
                size: data.size,
                leverage: data.leverage,
                direction: data.direction.toGem(),
            )
        }
    }

    public init(type: AutocloseType, currencyFormatter: CurrencyFormatter = .usd) {
        let session = Self.session(for: type)
        let separator = NumberInput.format(.current).decimalSeparator

        self.type = type
        self.currencyFormatter = currencyFormatter
        self.session = session
        estimator = Self.estimator(for: type)
        input = AutocloseInput(
            type: type,
            takeProfitText: session.initialText(tpslType: .takeProfit, decimalSeparator: separator),
            stopLossText: session.initialText(tpslType: .stopLoss, decimalSeparator: separator),
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
                decimals: data.assetDecimals,
                provider: .hypercore,
            )
        }
    }

    public var title: String {
        Localized.Perpetual.autoClose
    }

    public var marketPriceField: ListItemField {
        ListItemField(title: Localized.Perpetual.marketPrice, value: viewState.marketPrice.text())
    }

    public var takeProfitModel: AutocloseViewModel {
        autocloseModel(type: .takeProfit, price: takeProfitPrice)
    }

    public var stopLossModel: AutocloseViewModel {
        autocloseModel(type: .stopLoss, price: stopLossPrice)
    }

    public var positionItemViewModel: any ListAssetItemViewable {
        switch type {
        case let .modify(position, _): PerpetualPositionItemViewModel(data: position)
        case let .open(data, _): OpenPositionItemViewModel(data: data)
        }
    }

    public var entryPriceField: ListItemField? {
        viewState.entryPrice.map { ListItemField(title: Localized.Perpetual.entryPrice, value: $0.text()) }
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
        input.update()
        onChangePrice()

        let attempted = session.onSubmitAttempt()
        session = attempted
        guard attempted.viewState().confirmEnabled else { return }

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
        focused.text = perpetualFormatter.formatInputPrice(
            estimator.targetPriceFromRoe(roePercent: Int32(percent), triggerType: type.toGem()),
            decimals: assetDecimals,
        )
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

    private var position: PerpetualPositionData? {
        guard case let .modify(position, _) = type else { return nil }
        return position
    }

    private var marketPrice: Double {
        switch type {
        case let .modify(position, _): position.perpetual.price
        case let .open(data, _): data.marketPrice
        }
    }

    private var entryPrice: Double? {
        switch type {
        case let .modify(position, _): position.position.entryPrice
        case .open: nil
        }
    }

    private var assetDecimals: Int32 {
        switch type {
        case let .modify(position, _): position.asset.decimals
        case let .open(data, _): data.assetDecimals
        }
    }

    private func autocloseModel(type: TpslType, price: Double?) -> AutocloseViewModel {
        AutocloseViewModel(type: type, price: price, estimator: estimator)
    }
}
