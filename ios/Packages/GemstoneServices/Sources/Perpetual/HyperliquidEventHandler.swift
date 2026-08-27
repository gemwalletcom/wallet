// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public actor HyperliquidEventHandler {
    private let perpetualService: HyperliquidPerpetualServiceable
    private let chartService: any ChartStreamable

    public init(
        perpetualService: HyperliquidPerpetualServiceable,
        chartService: any ChartStreamable,
    ) {
        self.perpetualService = perpetualService
        self.chartService = chartService
    }

    public func handle(_ data: Data, walletId: WalletId, mode: PerpetualAccountMode) async {
        do {
            switch try await perpetualService.applySocketMessage(walletId: walletId, mode: mode, data: data) {
            case .applied: break
            case let .candle(candle):
                try await chartService.yield(Primitives.ChartCandleUpdate(candle))
            case let .subscriptionResponse(subscriptionType):
                debugLog("HyperliquidEventHandler: subscription response - \(subscriptionType)")
            case let .error(message):
                debugLog("HyperliquidEventHandler: error message: \(message)")
            case .unknown:
                debugLog("HyperliquidEventHandler: unknown message: \(String(data: data, encoding: .utf8) ?? "nil")")
            }
        } catch {
            debugLog("HyperliquidEventHandler: handle message error: \(error)")
        }
    }
}
