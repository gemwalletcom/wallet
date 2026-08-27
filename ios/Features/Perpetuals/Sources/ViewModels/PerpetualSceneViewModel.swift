// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import BigInt
import protocol Gemstone.GemExplorerServiceProtocol
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
    private let observerService: any PerpetualObservable
    private let transactionsService: any GemTransactionsServiceProtocol
    private let onTransferData: TransferDataAction
    private let onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)?
    private let perpetualOrderFactory = PerpetualOrderFactory()
    private let balanceCalculator = BalanceCalculator()

    public let wallet: Wallet
    public let asset: Asset

    public let explorerService: any GemExplorerServiceProtocol

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
    public var isPresentingAutoclose: Bool = false

    private let preferencesService: any GemPreferencesServiceProtocol

    public init(
        wallet: Wallet,
        asset: Asset,
        perpetualService: PerpetualServiceable,
        transactionsService: any GemTransactionsServiceProtocol,
        observerService: any PerpetualObservable,
        explorerService: any GemExplorerServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        onTransferData: TransferDataAction = nil,
        onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.transactionsService = transactionsService
        self.observerService = observerService
        self.explorerService = explorerService
        self.preferencesService = preferencesService
        chart = PerpetualChartModel(
            perpetualService: perpetualService,
            observerService: observerService,
            preferencesService: preferencesService,
        )
        self.onTransferData = onTransferData
        self.onPerpetualRecipientData = onPerpetualRecipientData

        positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: wallet.id, filter: .assetId(asset.id)), initialValue: [])
        perpetualQuery = ObservableQuery(PerpetualRequest(assetId: asset.id), initialValue: .empty)
        perpetualFiatValuesQuery = ObservableQuery(AssetFiatValuesRequest(walletId: wallet.id, type: .perpetual), initialValue: [])
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
        preferencesService.currencyCode
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
    func fetch() async {
        async let updateObserver: PerpetualAccountMode? = observerService.update(for: wallet)
        async let refreshTransactions: () = updateTransactions()
        async let refreshCandlesticks: () = chart.refresh(symbol: perpetual.coin)
        _ = await (updateObserver, refreshTransactions, refreshCandlesticks)
    }

    func onAppear() async {
        async let refresh: () = fetch()
        await chart.onAppear(symbol: perpetual.coin)
        await subscribeMarket(perpetual.coin)
        _ = await refresh
    }

    func onDisappear() async {
        await chart.onDisappear(symbol: perpetual.coin)
        await unsubscribeMarket(perpetual.coin)
    }

    func onScenePhaseChange(_: ScenePhase, _ newPhase: ScenePhase) {
        switch newPhase {
        case .active:
            Task { await updateTransactions() }
            Task { await chart.refresh(symbol: perpetual.coin) }
        case .inactive, .background: break
        @unknown default: break
        }
    }

    func onPeriodChange(_ oldPeriod: ChartPeriod, _ newPeriod: ChartPeriod) {
        Task {
            await chart.onPeriodChange(symbol: perpetual.coin, from: oldPeriod, to: newPeriod)
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
        isPresentingAutoclose = true
    }

    func onSelectAutocloseInfo() {
        isPresentingInfoSheet = .autoclose
    }

    func onModifyPosition() {
        isPresentingModifyAlert = true
    }

    func onClosePosition() {
        guard
            let position = positions.first?.position,
            let assetIndex = UInt32(perpetual.identifier)
        else { return }

        let data = perpetualOrderFactory.makeCloseOrder(
            assetIndex: Int32(assetIndex),
            perpetual: perpetual,
            position: position,
            asset: asset,
            baseAsset: Chain.hyperCore.defaultAsset(type: .perpetual),
        )

        let transferData = TransferData(
            type: .perpetual(asset, .close(data)),
            recipient: .hyperliquidProvider,
            value: .zero,
        )

        onTransferData?(transferData)
    }

    func onOpenLongPosition() {
        guard let transferData = createTransferData(
            direction: .long,
            leverage: perpetual.maxLeverage,
            marginType: perpetual.marginType,
        ) else {
            return
        }
        onPositionAction(.open(transferData))
    }

    func onOpenShortPosition() {
        guard let transferData = createTransferData(
            direction: .short,
            leverage: perpetual.maxLeverage,
            marginType: perpetual.marginType,
        ) else {
            return
        }
        onPositionAction(.open(transferData))
    }

    func onIncreasePosition() {
        isPresentingModifyAlert = false

        guard let position = positions.first?.position,
              let transferData = createTransferData(direction: position.direction, leverage: position.leverage, marginType: position.marginType)
        else { return }

        onPositionAction(.increase(transferData))
    }

    func onReducePosition() {
        isPresentingModifyAlert = false

        guard let position = positions.first?.position else {
            return
        }

        let direction: PerpetualDirection = switch position.direction {
        case .long: .short
        case .short: .long
        }

        guard let transferData = createTransferData(direction: direction, leverage: position.leverage, marginType: position.marginType) else {
            return
        }
        let baseAsset = Chain.hyperCore.defaultAsset(type: .perpetual)

        onPositionAction(
            .reduce(
                transferData,
                available: BigInt(position.marginAmount * pow(10.0, Double(baseAsset.decimals))),
                positionDirection: position.direction,
            ),
        )
    }

    func onAutocloseComplete() {
        isPresentingAutoclose = false
    }
}

// MARK: - Private

private extension PerpetualSceneViewModel {
    func subscribeMarket(_ coin: String) async {
        do {
            try await observerService.subscribe(.marketData(symbol: coin))
        } catch {
            debugLog("Market data subscription failed: \(error)")
        }
    }

    func unsubscribeMarket(_ coin: String) async {
        do {
            try await observerService.unsubscribe(.marketData(symbol: coin))
        } catch {
            debugLog("Market data unsubscribe failed: \(error)")
        }
    }

    func createTransferData(direction: PerpetualDirection, leverage: UInt8, marginType: PerpetualMarginType) -> PerpetualTransferData? {
        guard let assetIndex = Int(perpetual.identifier) else {
            return nil
        }

        return PerpetualTransferData(
            provider: perpetual.provider,
            direction: direction,
            asset: asset,
            baseAsset: Chain.hyperCore.defaultAsset(type: .perpetual),
            assetIndex: assetIndex,
            price: perpetual.price,
            leverage: leverage,
            marginType: marginType,
        )
    }

    func onPositionAction(_ positionAction: PerpetualPositionAction) {
        let recipientData = PerpetualRecipientData(
            recipient: .hyperliquid(),
            positionAction: positionAction,
        )
        onPerpetualRecipientData?(recipientData)
    }

    func updateTransactions() async {
        do {
            try await transactionsService.sync(walletId: wallet.id.id, assetId: asset.id.identifier)
        } catch {
            debugLog("perpetual scene: fetchTransactions error \(error)")
        }
    }
}
