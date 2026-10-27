// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.candleSession
import struct Gemstone.GemCandleChart
import struct Gemstone.GemCandleSession
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import struct Gemstone.GemTransactionRow
import struct Gemstone.GemTransactionsRequest
import struct Gemstone.PerpetualPosition
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class PerpetualChartViewModel {
    private let service: any GemPerpetualDetailsServiceProtocol
    private let observerService: any PerpetualObservable
    private let walletId: WalletId
    private let asset: Asset

    private var session: GemCandleSession {
        didSet { markersQuery.setRequestInBackground(Self.markersRequest(walletId: walletId, session.markersRequest(assetId: asset.id))) }
    }

    public let markersQuery: ObservableQuery<MappedQuery<TransactionsQuery, [GemTransactionRow]>>

    public var isPinching = false

    public var currentPeriod: ChartPeriod {
        get { session.period.toPrimitives() }
        set { session = session.onSelectPeriod(period: newValue.toGem()) }
    }

    public init(service: any GemPerpetualDetailsServiceProtocol, observerService: any PerpetualObservable, walletId: WalletId, asset: Asset) {
        self.service = service
        self.observerService = observerService
        self.walletId = walletId
        self.asset = asset
        let session = candleSession(period: service.chartPeriod())
        self.session = session
        markersQuery = ObservableQuery(Self.markersRequest(walletId: walletId, session.markersRequest(assetId: asset.id)), initialValue: [])
    }

    public func state(position: PerpetualPosition?) -> StateViewType<GemCandleChart> {
        session.viewState().state.stateViewType(session.chart(asset: asset.toGem(), position: position, transactions: markersQuery.value, utcOffsetSeconds: Int32(TimeZone.current.secondsFromGMT())))
    }
}

// MARK: - Actions

public extension PerpetualChartViewModel {
    func observe(perpetual: Perpetual) async {
        observerService.subscribe(candleSubscription(perpetual: perpetual, period: currentPeriod))
        await observeCandles()
        observerService.unsubscribe(candleSubscription(perpetual: perpetual, period: currentPeriod))
    }

    func onPeriodChange(perpetual: Perpetual, from oldPeriod: ChartPeriod, to newPeriod: ChartPeriod) {
        observerService.unsubscribe(candleSubscription(perpetual: perpetual, period: oldPeriod))
        observerService.subscribe(candleSubscription(perpetual: perpetual, period: newPeriod))
    }

    func refresh(perpetual: Perpetual) async {
        session = session.onRefresh()
        await loadCandles(perpetual: perpetual)
    }

    func loadCandles(perpetual: Perpetual) async {
        session = session.onSelectMarket(perpetual: perpetual.toGem())
        guard let request = session.request() else { return }
        let result = await service.candles(request: request)
        session = session.onResult(result: result)
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
    static func markersRequest(walletId: WalletId, _ request: GemTransactionsRequest) -> MappedQuery<TransactionsQuery, [GemTransactionRow]> {
        MappedQuery(TransactionsQuery(walletId: walletId, type: .all, filter: request.filter.toPrimitives(), limit: request.limit), transform: transactionListRows)
    }

    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        service.candleSubscription(perpetual: perpetual.toGem(), period: period.toGem())
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
