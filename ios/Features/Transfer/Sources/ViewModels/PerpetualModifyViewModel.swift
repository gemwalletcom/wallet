// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAutocloseSummary
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives

public struct PerpetualModifyViewModel: Sendable {
    private let summary: GemAutocloseSummary?
    private let autocloseFormatter = AutocloseFormatter(
        takeProfitLabel: Localized.Perpetual.takeProfit,
        stopLossLabel: Localized.Perpetual.stopLoss,
    )

    public init(summary: GemAutocloseSummary?) {
        self.summary = summary
    }

    public var listItemModel: ListItemModel? {
        guard let summary else {
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
