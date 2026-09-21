// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
public final class PerpetualChartModel {
    private let service: any GemPerpetualDetailsServiceProtocol
    private let observerService: any PerpetualObservable

    private var observeTask: Task<Void, Never>?

    public var state: StateViewType<PerpetualCandles> = .loading
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
}

// MARK: - Actions

public extension PerpetualChartModel {
    func onAppear(perpetual: Perpetual) async {
        await subscribeCandles(candleSubscription(perpetual: perpetual, period: currentPeriod))
        observeTask?.cancel()
        observeTask = Task {
            await observeCandles(perpetual: perpetual)
        }
    }

    func onDisappear(perpetual: Perpetual) async {
        observeTask?.cancel()
        observeTask = nil
        await unsubscribeCandles(candleSubscription(perpetual: perpetual, period: currentPeriod))
    }

    func onPeriodChange(perpetual: Perpetual, from oldPeriod: ChartPeriod, to newPeriod: ChartPeriod) async {
        state = .loading
        await unsubscribeCandles(candleSubscription(perpetual: perpetual, period: oldPeriod))
        await updateCandlesticks(perpetual: perpetual)
        await subscribeCandles(candleSubscription(perpetual: perpetual, period: newPeriod))
    }

    func refresh(perpetual: Perpetual) async {
        await updateCandlesticks(perpetual: perpetual)
    }
}

// MARK: - Private

private extension PerpetualChartModel {
    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        service.candleSubscription(perpetual: perpetual, period: period)
    }

    func updateCandlesticks(perpetual: Perpetual) async {
        let period = currentPeriod
        if state.value == nil {
            state = .loading
        }
        do {
            let candlesticks = try await service.candlesticks(perpetual: perpetual, period: period)
            guard period == currentPeriod else { return }
            state = .data(PerpetualCandles(period: period, candles: candlesticks))
        } catch {
            guard period == currentPeriod else { return }
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

    func observeCandles(perpetual: Perpetual) async {
        for await update in await observerService.chartService.makeStream() {
            if Task.isCancelled {
                break
            }
            mergeCandle(update, perpetual: perpetual)
        }
    }

    func mergeCandle(_ update: ChartCandleUpdate, perpetual: Perpetual) {
        guard case let .data(loaded) = state,
              let merged = service.mergedCandles(update: update, into: loaded.candles, perpetual: perpetual, period: loaded.period)
        else {
            return
        }
        state = .data(PerpetualCandles(period: loaded.period, candles: merged))
    }
}
