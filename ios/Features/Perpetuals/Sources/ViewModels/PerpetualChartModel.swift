// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.candleSession
import struct Gemstone.GemCandleSession
import struct Gemstone.GemCandleViewState
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

    private var session: GemCandleSession

    public var isPinching = false

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

    public var state: StateViewType<GemCandleViewState> {
        let viewState = session.viewState()
        return viewState.state.stateViewType(viewState.viewport.candles).map { _ in viewState }
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
            await observeCandles()
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

    func onZoom(_ magnification: Double) {
        session = session.onZoom(magnification: magnification)
    }
}

// MARK: - Private

private extension PerpetualChartModel {
    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        service.candleSubscription(perpetual: perpetual, period: period)
    }

    func updateCandlesticks(perpetual: Perpetual) async {
        session = session.onSelectMarket(perpetual: perpetual.toGem())
        guard let request = session.request() else { return }
        let result = await service.candles(request: request)
        session = session.onResult(result: result)
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

    func observeCandles() async {
        for await update in await observerService.chartService.makeStream() {
            if Task.isCancelled {
                break
            }
            session = session.onCandleUpdate(update: update.toGem())
        }
    }
}
