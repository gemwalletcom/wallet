// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.candleSession
import struct Gemstone.GemCandleChart
import struct Gemstone.GemCandleSession
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import struct Gemstone.PerpetualPosition
import GemstonePrimitives
import GemstoneServices
import Primitives
import SwiftUI

@Observable
@MainActor
public final class PerpetualChartViewModel {
    private let service: any GemPerpetualDetailsServiceProtocol
    private let observerService: any PerpetualObservable
    private let asset: Asset

    private var observeTask: Task<Void, Never>?

    private var session: GemCandleSession

    public var isPinching = false

    public var currentPeriod: ChartPeriod {
        get { session.period.toPrimitives() }
        set { session = session.onSelectPeriod(period: newValue.toGem()) }
    }

    public init(service: any GemPerpetualDetailsServiceProtocol, observerService: any PerpetualObservable, asset: Asset) {
        self.service = service
        self.observerService = observerService
        self.asset = asset
        session = candleSession(period: service.chartPeriod())
    }

    public func state(position: PerpetualPosition?) -> StateViewType<GemCandleChart> {
        session.viewState().state.stateViewType(session.chart(asset: asset.toGem(), position: position, utcOffsetSeconds: Int32(TimeZone.current.secondsFromGMT())))
    }
}

// MARK: - Actions

public extension PerpetualChartViewModel {
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
        session = session.onRefresh()
        await updateCandlesticks(perpetual: perpetual)
    }

    func onZoom(_ magnification: Double, anchor: Double) {
        let zoomed = session.onZoom(magnification: magnification, anchor: anchor)
        guard zoomed.zoom != session.zoom else { return }
        session = zoomed
    }

    func onPan(_ fraction: Double) {
        let panned = session.onPan(fraction: fraction)
        guard panned.zoom != session.zoom else { return }
        session = panned
    }
}

// MARK: - Private

private extension PerpetualChartViewModel {
    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        service.candleSubscription(perpetual: perpetual.toGem(), period: period.toGem())
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
