// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.candleSession
import struct Gemstone.GemCandleSession
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
public final class PerpetualChartViewModel {
    private let service: any GemPerpetualDetailsServiceProtocol
    private let observerService: any PerpetualObservable

    private var observeTask: Task<Void, Never>?

    private var session: GemCandleSession

    public var currentPeriod: ChartPeriod {
        get { session.period.toPrimitives() }
        set {
            session = session.onSelectPeriod(period: newValue.toGem())
            service.setChartPeriodValue(newValue)
        }
    }

    public init(service: any GemPerpetualDetailsServiceProtocol, observerService: any PerpetualObservable) {
        self.service = service
        self.observerService = observerService
        session = candleSession(period: service.chartPeriodValue.toGem())
    }

    public var state: StateViewType<PerpetualCandles> {
        let viewState = session.viewState()
        return viewState.state.stateViewType(PerpetualCandles(period: viewState.period.toPrimitives(), candles: viewState.candles.map { $0.toPrimitives() }))
    }

    public var emptyTitle: String { Localized.Common.notAvailable }
    public var emptyImage: Image { Images.EmptyContent.activity }
}

// MARK: - Actions

public extension PerpetualChartViewModel {
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
        session = session.onRefresh()
        await updateCandlesticks(perpetual: perpetual)
    }
}

// MARK: - Private

private extension PerpetualChartViewModel {
    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        service.candleSubscription(perpetual: perpetual, period: period)
    }

    func updateCandlesticks(perpetual: Perpetual) async {
        session = session.onSelectMarket(perpetual: perpetual.toGem())
        guard let request = session.request() else { return }
        session = await session.onResult(result: service.candles(request: request))
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
        let viewState = session.viewState()
        guard let merged = service.mergedCandles(update: update, into: viewState.candles.map { $0.toPrimitives() }, perpetual: perpetual, period: viewState.period.toPrimitives()) else { return }
        session = session.onCandles(candles: merged.map { $0.toGem() })
    }
}
