// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
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
    private let service: any GemPerpetualDetailsServiceProtocol
    private let observerService: any PerpetualObservable

    private var observeTask: Task<Void, Never>?

    public var state: StateViewType<[ChartCandleStick]> = .loading
    public var currentPeriod: ChartPeriod {
        didSet { service.setChartPeriodValue(currentPeriod) }
    }

    public init(service: any GemPerpetualDetailsServiceProtocol, observerService: any PerpetualObservable) {
        self.service = service
        self.observerService = observerService
        currentPeriod = service.chartPeriodValue
    }

    public var emptyTitle: String { Localized.Common.notAvailable }
    public var emptyImage: Image { Images.EmptyContent.activity }
    private var currentInterval: String { service.candleInterval(period: currentPeriod.json()) }
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
        service.candleSubscription(symbol: symbol, period: period)
    }

    func updateCandlesticks(symbol: String) async {
        state = .loading
        do {
            let candlesticks = try await service.candlesticks(symbol: symbol, period: currentPeriod)
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
            do {
                try handleChartUpdate(update, symbol: symbol)
            } catch {
                debugLog("Chart update failed: \(error)")
            }
        }
    }

    func handleChartUpdate(_ update: ChartCandleUpdate, symbol: String) throws {
        guard update.coin == symbol,
              update.interval == currentInterval,
              case let .data(candlesticks) = state
        else {
            return
        }

        state = try .data(service.merge(candlesticks: candlesticks, candle: update.candle))
    }
}
