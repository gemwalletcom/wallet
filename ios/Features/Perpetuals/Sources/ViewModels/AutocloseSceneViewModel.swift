// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AutocloseSceneViewModel {
    private let currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)
    private let percentFormatter = CurrencyFormatter(type: .percent, currencyCode: Currency.usd.rawValue)
    private let perpetualFormatter = PerpetualFormatter(provider: .hypercore)
    private let type: AutocloseType
    private let estimator: AutocloseEstimator

    var input: AutocloseInput

    public init(type: AutocloseType) {
        self.type = type
        estimator = AutocloseEstimator(type: type)

        input = AutocloseInput(
            type: type,
            takeProfitText: Self.initialText(for: .takeProfit, type: type),
            stopLossText: Self.initialText(for: .stopLoss, type: type),
        )
    }

    public var title: String { Localized.Perpetual.autoClose }
    public var marketPriceField: ListItemField {
        ListItemField(title: Localized.Perpetual.marketPrice, value: currencyFormatter.string(marketPrice))
    }

    public var takeProfitModel: AutocloseViewModel { autocloseModel(type: .takeProfit, price: takeProfitPrice) }
    public var stopLossModel: AutocloseViewModel { autocloseModel(type: .stopLoss, price: stopLossPrice) }

    public var positionItemViewModel: any ListAssetItemViewable {
        switch type {
        case let .modify(position, _): PerpetualPositionItemViewModel(model: PerpetualPositionViewModel(position))
        case let .open(data, _): OpenPositionItemViewModel(data: data)
        }
    }

    public var entryPriceField: ListItemField? {
        switch type {
        case let .modify(position, _):
            ListItemField(title: Localized.Perpetual.entryPrice, value: currencyFormatter.string(position.position.entryPrice))
        case .open: nil
        }
    }

    public var confirmButtonType: ButtonType {
        let builder = AutocloseModifyBuilder(direction: type.direction)
        let isEnabled = builder.canBuild(takeProfit: takeProfitField, stopLoss: stopLossField)
        return .primary(isEnabled ? .normal : .disabled)
    }
}

// MARK: - Actions

public extension AutocloseSceneViewModel {
    func onChangeFocusField(_ _: AutocloseScene.Field?, _ newField: AutocloseScene.Field?) {
        input.focusField = newField
    }

    func onSelectConfirm() {
        input.update()

        let builder = AutocloseModifyBuilder(direction: type.direction)
        guard builder.canBuild(takeProfit: takeProfitField, stopLoss: stopLossField) else { return }

        switch type {
        case let .modify(position, onTransferAction):
            guard let assetIndex = Int32(position.perpetual.identifier) else { return }

            let modifyTypes = builder.build(
                assetIndex: assetIndex,
                takeProfit: takeProfitField,
                stopLoss: stopLossField,
            )

            let data = PerpetualModifyConfirmData(
                baseAsset: .hypercoreUSDC(),
                assetIndex: assetIndex,
                modifyTypes: modifyTypes,
                takeProfitOrderId: takeProfitOrderId,
                stopLossOrderId: stopLossOrderId,
            )

            onTransferAction?(
                TransferData(
                    type: .perpetual(position.asset, .modify(data)),
                    recipientData: .hyperliquid(),
                    value: .zero,
                    canChangeValue: false,
                ),
            )

        case let .open(_, onComplete):
            onComplete(input.takeProfit, input.stopLoss)
        }
    }

    func onSelectPercent(_ percent: Int) {
        guard let type = input.focusedType, let focused = input.focused else { return }
        focused.text = perpetualFormatter.formatInputPrice(
            estimator.calculateTargetPriceFromROE(roePercent: percent, type: type),
            decimals: assetDecimals,
        )
    }
}

// MARK: - Private

extension AutocloseSceneViewModel {
    private var takeProfitPrice: Double? { currencyFormatter.double(from: input.takeProfit.text) }
    private var stopLossPrice: Double? { currencyFormatter.double(from: input.stopLoss.text) }

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

    private var assetDecimals: Int32 {
        switch type {
        case let .modify(position, _): position.asset.decimals
        case let .open(data, _): data.assetDecimals
        }
    }

    private var takeProfitField: AutocloseField {
        let price: Double? = switch type {
        case let .modify(position, _): position.position.takeProfit?.price
        case let .open(data, _): data.takeProfit.flatMap { currencyFormatter.double(from: $0) }
        }
        return input.field(
            type: .takeProfit,
            price: takeProfitPrice,
            originalPrice: price,
            formattedPrice: takeProfitPrice.map { formatPrice($0) },
            orderId: takeProfitOrderId,
        )
    }

    private var stopLossField: AutocloseField {
        let price: Double? = switch type {
        case let .modify(position, _): position.position.stopLoss?.price
        case let .open(data, _): data.stopLoss.flatMap { currencyFormatter.double(from: $0) }
        }
        return input.field(
            type: .stopLoss,
            price: stopLossPrice,
            originalPrice: price,
            formattedPrice: stopLossPrice.map { formatPrice($0) },
            orderId: stopLossOrderId,
        )
    }

    private func autocloseModel(type: TpslType, price: Double?) -> AutocloseViewModel {
        AutocloseViewModel(
            type: type,
            price: price,
            estimator: estimator,
            currencyFormatter: currencyFormatter,
            percentFormatter: percentFormatter,
        )
    }

    private var takeProfitOrderId: UInt64? {
        guard let orderId = position?.position.takeProfit?.order_id else { return nil }
        return UInt64(orderId)
    }

    private var stopLossOrderId: UInt64? {
        guard let orderId = position?.position.stopLoss?.order_id else { return nil }
        return UInt64(orderId)
    }

    private func formatPrice(_ price: Double) -> String {
        perpetualFormatter.formatPrice(price, decimals: assetDecimals)
    }

    private static func initialText(for tpslType: TpslType, type: AutocloseType) -> String? {
        switch type {
        case let .modify(position, _):
            let tpsl = tpslType == .takeProfit ? position.position.takeProfit : position.position.stopLoss
            return tpsl.map { PerpetualFormatter(provider: .hypercore).formatInputPrice($0.price, decimals: position.asset.decimals) }
        case let .open(data, _):
            return tpslType == .takeProfit ? data.takeProfit : data.stopLoss
        }
    }
}
