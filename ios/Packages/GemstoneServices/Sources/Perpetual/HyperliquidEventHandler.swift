// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemHyperliquidOpenOrder
import class Gemstone.Hyperliquid
import Primitives

public actor HyperliquidEventHandler {
    private let perpetualService: HyperliquidPerpetualServiceable
    private let chartService: any ChartStreamable
    private let hyperliquid = Hyperliquid()

    public init(
        perpetualService: HyperliquidPerpetualServiceable,
        chartService: any ChartStreamable,
    ) {
        self.perpetualService = perpetualService
        self.chartService = chartService
    }

    public func handle(_ data: Data, walletId: WalletId, mode: PerpetualAccountMode) async {
        do {
            switch try hyperliquid.parseWebsocketData(data: data, mode: mode.json()) {
            case let .accountState(balance, newPositions):
                try await handleAccountState(
                    walletId: walletId,
                    balance: balance.map { try Primitives.PerpetualBalance($0) },
                    newPositions: newPositions.map { try Primitives.PerpetualPosition($0) },
                )
            case let .spotState(balance):
                try await perpetualService.updateBalance(walletId: walletId, balance: Primitives.PerpetualBalance(balance))
            case let .openOrders(orders):
                try await handleOpenOrders(walletId: walletId, orders: orders)
            case let .candle(candle):
                try await chartService.yield(Primitives.ChartCandleUpdate(candle))
            case let .marketData(market):
                try await perpetualService.updateMarket(Primitives.PerpetualMarketData(market))
            case let .marketPrices(prices):
                try await perpetualService.updatePrices(prices)
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

    // MARK: - Private

    private func handleAccountState(
        walletId: WalletId,
        balance: Primitives.PerpetualBalance?,
        newPositions: [Primitives.PerpetualPosition],
    ) async throws {
        let diff = try await hyperliquid.diffClearinghousePositions(
            newPositions: newPositions.map { try $0.json() },
            existingPositions: perpetualService.getHypercorePositions(walletId: walletId).map { try $0.json() },
        )
        try await perpetualService.updatePositions(
            walletId: walletId,
            positions: diff.positions.map { try Primitives.PerpetualPosition($0) },
            deleteIds: diff.deletePositionIds,
        )
        if let balance {
            try await perpetualService.updateBalance(walletId: walletId, balance: balance)
        }
    }

    private func handleOpenOrders(walletId: WalletId, orders: [GemHyperliquidOpenOrder]) async throws {
        let diff = try await hyperliquid.diffOpenOrdersPositions(
            orders: orders,
            existingPositions: perpetualService.getHypercorePositions(walletId: walletId).map { try $0.json() },
        )
        try await perpetualService.updatePositions(
            walletId: walletId,
            positions: diff.positions.map { try Primitives.PerpetualPosition($0) },
            deleteIds: diff.deletePositionIds,
        )
    }
}
