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
        state = .loading
        do {
            let candlesticks = try await service.candlesticks(perpetual: perpetual, period: currentPeriod)
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

    func observeCandles(perpetual: Perpetual) async {
        for await update in await observerService.chartService.makeStream() {
            if Task.isCancelled { break }
            do {
                try handleChartUpdate(update, perpetual: perpetual)
            } catch {
                debugLog("Chart update failed: \(error)")
            }
        }
    }

    func handleChartUpdate(_ update: ChartCandleUpdate, perpetual: Perpetual) throws {
        guard case let .data(candlesticks) = state,
              let merged = try service.apply(update: update, to: candlesticks, perpetual: perpetual, period: currentPeriod)
        else {
            return
        }
        state = .data(merged)
    }
}
