// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import Components
import Foundation
import enum Gemstone.GemPerpetualSubscription
import Localization
import GemstoneServices
import Preferences
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
public final class PerpetualChartModel {
    private let perpetualService: any GemPerpetualServiceProtocol
    private let observerService: any PerpetualObservable
    private let preferencesService: any GemPreferencesServiceProtocol

    private var observeTask: Task<Void, Never>?

    public var state: StateViewType<[ChartCandleStick]> = .loading
    public var currentPeriod: ChartPeriod {
        didSet { preferencesService.setPerpetualChartPeriodValue(currentPeriod) }
    }

    public init(
        perpetualService: any GemPerpetualServiceProtocol,
        observerService: any PerpetualObservable,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.perpetualService = perpetualService
        self.observerService = observerService
        self.preferencesService = preferencesService
        currentPeriod = preferencesService.perpetualChartPeriodValue
    }

    public var emptyTitle: String { Localized.Common.notAvailable }
    public var emptyImage: Image { Images.EmptyContent.activity }
    private var currentInterval: String { currentPeriod.hyperliquidInterval }
}

// MARK: - Actions

public extension PerpetualChartModel {
    func onAppear(symbol: String) async {
        await subscribeCandles(candleSubscription(symbol: symbol, period: currentPeriod))
        observeTask?.cancel()
        observeTask = Task {
            await observeCandles(symbol: symbol)
        }
    }

    func onDisappear(symbol: String) async {
        observeTask?.cancel()
        observeTask = nil
        await unsubscribeCandles(candleSubscription(symbol: symbol, period: currentPeriod))
    }

    func onPeriodChange(symbol: String, from oldPeriod: ChartPeriod, to newPeriod: ChartPeriod) async {
        await unsubscribeCandles(candleSubscription(symbol: symbol, period: oldPeriod))
        await updateCandlesticks(symbol: symbol)
        await subscribeCandles(candleSubscription(symbol: symbol, period: newPeriod))
    }

    func refresh(symbol: String) async {
        await updateCandlesticks(symbol: symbol)
    }
}

// MARK: - Private

private extension PerpetualChartModel {
    func candleSubscription(symbol: String, period: ChartPeriod) -> GemPerpetualSubscription {
        .candle(symbol: symbol, interval: period.hyperliquidInterval)
    }

    func updateCandlesticks(symbol: String) async {
        state = .loading
        do {
            let candlesticks = try await perpetualService.candlesticks(
                symbol: symbol,
                period: currentPeriod,
            )
            state = .data(candlesticks)
        } catch {
            state.setError(error)
        }
    }

    func subscribeCandles(_ subscription: GemPerpetualSubscription) async {
        do {
            try await observerService.subscribe(subscription)
        } catch {
            debugLog("Chart subscription failed: \(error)")
        }
    }

    func unsubscribeCandles(_ subscription: GemPerpetualSubscription) async {
        do {
            try await observerService.unsubscribe(subscription)
        } catch {
            debugLog("Chart unsubscribe failed: \(error)")
        }
    }

    func observeCandles(symbol: String) async {
        for await update in await observerService.chartService.makeStream() {
            if Task.isCancelled { break }
            handleChartUpdate(update, symbol: symbol)
        }
    }

    func handleChartUpdate(_ update: ChartCandleUpdate, symbol: String) {
        guard update.coin == symbol,
              update.interval == currentInterval,
              case var .data(candlesticks) = state,
              let lastCandle = candlesticks.last
        else {
            return
        }

        let candle = update.candle
        if lastCandle.date == candle.date {
            candlesticks[candlesticks.count - 1] = candle
        } else if candle.date > lastCandle.date {
            candlesticks.removeFirst()
            candlesticks.append(candle)
        }

        state = .data(candlesticks)
    }
}

private extension ChartPeriod {
    var hyperliquidInterval: String {
        switch self {
        case .hour: "1m"
        case .day: "30m"
        case .week: "4h"
        case .month: "12h"
        case .year: "1w"
        case .all: "1M"
        }
    }
}
