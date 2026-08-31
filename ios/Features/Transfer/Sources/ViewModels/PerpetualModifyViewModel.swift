// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemConfirmSceneServiceProtocol
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives

public struct PerpetualModifyViewModel: Sendable {
    private let data: PerpetualModifyConfirmData
    private let service: any GemConfirmSceneServiceProtocol
    private let autocloseFormatter = AutocloseFormatter(
        takeProfitLabel: Localized.Perpetual.takeProfit,
        stopLossLabel: Localized.Perpetual.stopLoss,
    )

    public init(data: PerpetualModifyConfirmData, service: any GemConfirmSceneServiceProtocol) {
        self.data = data
        self.service = service
    }

    public var listItemModel: ListItemModel? {
        guard let summary = service.autocloseSummary(data: data.json()) else {
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
