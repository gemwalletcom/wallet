// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitives
import Localization
import Primitives

public struct PerpetualModifyViewModel: Sendable {
    private let data: PerpetualModifyConfirmData
    private let perpetualService: any GemPerpetualServiceProtocol
    private let autocloseFormatter = AutocloseFormatter(
        takeProfitLabel: Localized.Perpetual.takeProfit,
        stopLossLabel: Localized.Perpetual.stopLoss,
    )

    public init(data: PerpetualModifyConfirmData, perpetualService: any GemPerpetualServiceProtocol) {
        self.data = data
        self.perpetualService = perpetualService
    }

    public var listItemModel: ListItemModel? {
        guard let summary = perpetualService.autocloseSummary(data: data.json()) else {
            return nil
        }
        let autoclose = autocloseFormatter.format(
            takeProfit: summary.takeProfit,
            stopLoss: summary.stopLoss,
            takeProfitCanceled: summary.takeProfitCleared,
            stopLossCanceled: summary.stopLossCleared,
        )

        return ListItemModel(
            title: Localized.Perpetual.autoClose,
            subtitle: autoclose.subtitle,
            subtitleExtra: autoclose.subtitleExtra,
        )
    }
}
