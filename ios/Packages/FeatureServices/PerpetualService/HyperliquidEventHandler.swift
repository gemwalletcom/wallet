// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemHyperliquidOpenOrder
import struct Gemstone.GemPerpetualBalance
import struct Gemstone.GemPerpetualPosition
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
            switch try hyperliquid.parseWebsocketData(data: data, mode: mode.map()) {
            case let .accountState(balance, newPositions):
                try handleAccountState(walletId: walletId, balance: balance, newPositions: newPositions)
            case let .spotState(balance):
                try updateBalance(walletId: walletId, balance: balance)
            case let .openOrders(orders):
                try handleOpenOrders(walletId: walletId, orders: orders)
            case let .candle(candle):
                await chartService.yield(candle.map())
            case let .marketData(market):
                try perpetualService.updateMarket(market)
            case let .marketPrices(prices):
                try perpetualService.updatePrices(prices)
            case let .subscriptionResponse(subscriptionType):
                debugLog("HyperliquidEventHandler: subscription response - \(subscriptionType)")
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
        balance: GemPerpetualBalance?,
        newPositions: [GemPerpetualPosition],
    ) throws {
        let diff = try hyperliquid.diffClearinghousePositions(
            newPositions: newPositions,
            existingPositions: perpetualService.getHypercorePositions(walletId: walletId),
        )

        try perpetualService.diffPositions(
            deleteIds: diff.deletePositionIds,
            positions: diff.positions,
            walletId: walletId,
        )
        if let balance {
            try updateBalance(walletId: walletId, balance: balance)
        }
    }

    private func updateBalance(walletId: WalletId, balance: GemPerpetualBalance) throws {
        try perpetualService.updateBalance(walletId: walletId, balance: balance)
    }

    private func handleOpenOrders(walletId: WalletId, orders: [GemHyperliquidOpenOrder]) throws {
        let diff = try hyperliquid.diffOpenOrdersPositions(
            orders: orders,
            existingPositions: perpetualService.getHypercorePositions(walletId: walletId),
        )
        try perpetualService.diffPositions(
            deleteIds: diff.deletePositionIds,
            positions: diff.positions,
            walletId: walletId,
        )
    }
}
