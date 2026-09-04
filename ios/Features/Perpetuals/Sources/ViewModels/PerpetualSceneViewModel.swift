// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualPositionKind
import BigInt
import Formatters
import Foundation
import GemstonePrimitives
import InfoSheet
import Localization
import GemstoneServices
import Preferences
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class PerpetualSceneViewModel {
    private let service: any GemPerpetualDetailsServiceProtocol
    private let observerService: any PerpetualObservable
    private let onTransferData: TransferDataAction
    private let onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)?
    private let balanceCalculator = BalanceCalculator()

    public let wallet: Wallet
    public let asset: Asset



    public let positionsQuery: ObservableQuery<PerpetualPositionsRequest>
    public let perpetualQuery: ObservableQuery<PerpetualRequest>
    public let perpetualFiatValuesQuery: ObservableQuery<AssetFiatValuesRequest>
    public let transactionsQuery: ObservableQuery<TransactionsRequest>

    public var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    public var perpetualData: PerpetualData {
        perpetualQuery.value
    }

    public var perpetualTotalValue: TotalFiatValue {
        balanceCalculator.totalFiatValue(perpetualFiatValuesQuery.value)
    }

    public var transactions: [TransactionExtended] {
        transactionsQuery.value
    }

    public let chart: PerpetualChartModel

    public var isPresentingInfoSheet: InfoSheetType?
    public var isPresentingModifyAlert: Bool?
    public var isPresentingAutoclose: PerpetualPositionData?

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemPerpetualDetailsServiceProtocol,
        observerService: any PerpetualObservable,
        onTransferData: TransferDataAction = nil,
        onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.service = service
        self.observerService = observerService
        chart = PerpetualChartModel(service: service, observerService: observerService)
        self.onTransferData = onTransferData
        self.onPerpetualRecipientData = onPerpetualRecipientData

        positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: wallet.id, filter: .assetId(asset.id)), initialValue: [])
        perpetualQuery = ObservableQuery(PerpetualRequest(assetId: asset.id), initialValue: .empty)
        perpetualFiatValuesQuery = ObservableQuery(
            AssetFiatValuesRequest(
                walletId: wallet.id,
                type: .perpetual,
                perpetualAssetId: Chain.hyperCore.defaultAsset(type: .perpetual).id,
            ),
            initialValue: [],
        )
        transactionsQuery = ObservableQuery(
            TransactionsRequest.perpetualScene(
                walletId: wallet.id,
                assetId: asset.id,
            ),
            initialValue: [],
        )
    }

    public var navigationTitle: String {
        let name = perpetualViewModel.name
        return name.isEmpty ? asset.symbol : name
    }

    public var currency: String {
        service.currency()
    }

    public var hasOpenPosition: Bool {
        !positionViewModels.isEmpty
    }

    public var positionSectionTitle: String {
        Localized.Perpetual.position
    }

    public var infoSectionTitle: String {
        Localized.Common.info
    }

    public var transactionsSectionTitle: String {
        Localized.Activity.title
    }

    public var closePositionTitle: String {
        Localized.Perpetual.closePosition
    }

    public var modifyPositionTitle: String {
        Localized.Perpetual.modify
    }

    public var increasePositionTitle: String {
        Localized.Perpetual.increasePosition
    }

    public var reducePositionTitle: String {
        Localized.Perpetual.reducePosition
    }

    public var longButtonTitle: String {
        Localized.Perpetual.long
    }

    public var shortButtonTitle: String {
        Localized.Perpetual.short
    }

    public var perpetual: Perpetual {
        perpetualData.perpetual
    }

    public var perpetualViewModel: PerpetualViewModel {
        PerpetualViewModel(perpetual: perpetual)
    }

    public var positionViewModels: [PerpetualPositionViewModel] {
        positions.map { PerpetualPositionViewModel($0) }
    }

    var chartLineModels: [ChartLineViewModel] {
        guard let positionData = positions.first else { return [] }
        let position = positionData.position
        let prices: [(ChartLineType, Double?)] = [
            (.entry, position.entryPrice),
            (.takeProfit, position.takeProfit?.price),
            (.stopLoss, position.stopLoss?.price),
            (.liquidation, position.liquidationPrice),
        ]
        return prices.compactMap { type, price in
            price.map {
                ChartLineViewModel(
                    line: ChartLine(type: type, price: $0),
                    formatter: NumericFormatter(),
                )
            }
        }
    }
}

// MARK: - Actions

public extension PerpetualSceneViewModel {
    func load() async {
        async let positions: () = syncPositions()
        async let refreshTransactions: () = updateTransactions()
        async let refreshCandlesticks: () = chart.refresh(perpetual: perpetual)
        _ = await (positions, refreshTransactions, refreshCandlesticks)
    }

    func onAppear() async {
        async let refresh: () = load()
        await chart.onAppear(perpetual: perpetual)
        await subscribeMarket()
        _ = await refresh
    }

    func onDisappear() async {
        await chart.onDisappear(perpetual: perpetual)
        await unsubscribeMarket()
    }

    func onScenePhaseChange(_: ScenePhase, _ newPhase: ScenePhase) {
        switch newPhase {
        case .active:
            Task { await updateTransactions() }
            Task { await chart.refresh(perpetual: perpetual) }
        case .inactive, .background: break
        @unknown default: break
        }
    }

    func onPeriodChange(_ oldPeriod: ChartPeriod, _ newPeriod: ChartPeriod) {
        Task {
            await chart.onPeriodChange(perpetual: perpetual, from: oldPeriod, to: newPeriod)
        }
    }

    func onSelectFundingRateInfo() {
        isPresentingInfoSheet = .fundingApr
    }

    func onSelectFundingPaymentsInfo() {
        isPresentingInfoSheet = .fundingPayments
    }

    func onSelectLiquidationPriceInfo() {
        isPresentingInfoSheet = .liquidationPrice
    }

    func onSelectOpenInterestInfo() {
        isPresentingInfoSheet = .openInterest
    }

    func onSelectAutoclose() {
        isPresentingAutoclose = positions.first
    }

    func onSelectAutocloseInfo() {
        isPresentingInfoSheet = .autoclose
    }

    func onModifyPosition() {
        isPresentingModifyAlert = true
    }

    func onClosePosition() {
        do {
            onTransferData?(try service.closeTransfer(perpetual: perpetual.json(), asset: asset.map(), position: positions.first?.position.json()))
        } catch {
            debugLog("perpetual scene: close position error \(error)")
        }
    }

    func onOpenLongPosition() {
        onPositionAction(.open(direction: PerpetualDirection.long.json()))
    }

    func onOpenShortPosition() {
        onPositionAction(.open(direction: PerpetualDirection.short.json()))
    }

    func onIncreasePosition() {
        isPresentingModifyAlert = false
        onPositionAction(.increase)
    }

    func onReducePosition() {
        isPresentingModifyAlert = false
        onPositionAction(.reduce)
    }

    func onAutocloseComplete() {
        isPresentingAutoclose = nil
    }
}

// MARK: - Private

private extension PerpetualSceneViewModel {
    func subscribeMarket() async {
        do {
            try await observerService.subscribe(service.marketSubscription(perpetual: perpetual.json()))
        } catch {
            debugLog("Market data subscription failed: \(error)")
        }
    }

    func unsubscribeMarket() async {
        do {
            try await observerService.unsubscribe(service.marketSubscription(perpetual: perpetual.json()))
        } catch {
            debugLog("Market data unsubscribe failed: \(error)")
        }
    }

    func onPositionAction(_ kind: GemPerpetualPositionKind) {
        do {
            let positionAction = try service.positionAction(perpetual: perpetual.json(), asset: asset.map(), position: positions.first?.position.json(), kind: kind)
            let recipientData = PerpetualRecipientData(
                recipient: RecipientData(recipient: PerpetualFormatter(provider: perpetual.provider).recipient, amount: .none),
                positionAction: positionAction,
            )
            onPerpetualRecipientData?(recipientData)
        } catch {
            debugLog("perpetual scene: position action error \(error)")
        }
    }

    func syncPositions() async {
        do {
            try await service.syncPositions()
        } catch {
            debugLog("perpetual scene: sync positions error \(error)")
        }
    }

    func updateTransactions() async {
        do {
            try await service.syncTransactions(assetId: asset.id.identifier)
        } catch {
            debugLog("perpetual scene: loadTransactions error \(error)")
        }
    }
}
