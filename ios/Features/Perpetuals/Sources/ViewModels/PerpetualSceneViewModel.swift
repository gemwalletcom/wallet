// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemPerpetualButton
import struct Gemstone.GemPerpetualButtonRow
import struct Gemstone.GemPerpetualDetails
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualPositionAction
import enum Gemstone.GemPerpetualPositionKind
import GemstonePrimitives
import GemstoneServices
import InfoSheet
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
    private let onPerpetualPosition: ((GemPerpetualPositionAction) -> Void)?

    public let wallet: Wallet
    public let asset: Asset

    public let positionsQuery: ObservableQuery<PerpetualPositionsQuery>
    public let perpetualQuery: ObservableQuery<PerpetualQuery>
    public let transactionsQuery: ObservableQuery<MappedQuery<TransactionsQuery, [ListSection<TransactionViewModel>]>>

    public var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    public var perpetualData: PerpetualData {
        perpetualQuery.value
    }

    public var transactionSections: [ListSection<TransactionViewModel>] {
        transactionsQuery.value
    }

    public let chart: PerpetualChartViewModel

    public var isPresentingInfoSheet: GemInfoSheet?
    public var isPresentingModifyAlert: Bool?
    public var isPresentingAutoclose: PerpetualPositionData?
    public var isPresentingAlertMessage: AlertMessage?

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemPerpetualDetailsServiceProtocol,
        observerService: any PerpetualObservable,
        onTransferData: TransferDataAction = nil,
        onPerpetualPosition: ((GemPerpetualPositionAction) -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.service = service
        self.observerService = observerService
        chart = PerpetualChartViewModel(service: service, observerService: observerService)
        self.onTransferData = onTransferData
        self.onPerpetualPosition = onPerpetualPosition

        positionsQuery = ObservableQuery(PerpetualPositionsQuery(walletId: wallet.id, filter: .assetId(asset.id)), initialValue: [])
        perpetualQuery = ObservableQuery(PerpetualQuery(assetId: asset.id), initialValue: .empty)
        transactionsQuery = ObservableQuery(
            MappedQuery(
                TransactionsQuery.perpetualScene(walletId: wallet.id, assetId: asset.id, types: GemConstants.perpetualActivityTypes, limit: GemConstants.transactionsListLimit),
                transform: TransactionViewModel.sections,
            ),
            initialValue: [],
        )
    }

    public var details: GemPerpetualDetails {
        service.details(perpetual: perpetual.toGem(), asset: asset.toGem(), positions: positions.map { $0.position.toGem() })
    }

    public var modifyTitle: String {
        GemPerpetualButton.modify.title
    }

    public func buttonModels(_ buttons: [GemPerpetualButtonRow]) -> [PerpetualButtonViewModel] {
        buttons.map { PerpetualButtonViewModel(row: $0) }
    }

    public func onSelect(_ button: PerpetualButtonViewModel) {
        onSelectButton(button.button)
    }

    public func onSelectButton(_ button: GemPerpetualButton) {
        switch button {
        case .long: onOpenLongPosition()
        case .short: onOpenShortPosition()
        case .modify: onModifyPosition()
        case .close: onClosePosition()
        case .increase: onIncreasePosition()
        case .reduce: onReducePosition()
        }
    }

    public var perpetual: Perpetual {
        perpetualData.perpetual
    }
}

// MARK: - Actions

public extension PerpetualSceneViewModel {
    func load() async {
        async let refresh: () = refreshStored()
        async let refreshCandlesticks: () = chart.refresh(perpetual: perpetual)
        _ = await (refresh, refreshCandlesticks)
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
            Task { await load() }
        case .inactive, .background: break
        @unknown default: break
        }
    }

    func onPeriodChange(_ oldPeriod: ChartPeriod, _ newPeriod: ChartPeriod) {
        Task {
            await chart.onPeriodChange(perpetual: perpetual, from: oldPeriod, to: newPeriod)
        }
    }

    func onInfo(_ topic: GemInfoTopic) {
        isPresentingInfoSheet = topic.infoSheet
    }

    func onSelectAutoclose() {
        isPresentingAutoclose = positionData(details)
    }

    func onModifyPosition() {
        isPresentingModifyAlert = true
    }

    func onClosePosition() {
        do {
            try onTransferData?(service.closeTransfer(perpetual: perpetual.toGem(), asset: asset.toGem(), position: details.position))
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onOpenLongPosition() {
        onPositionAction(.open(direction: PerpetualDirection.long.toGem()))
    }

    func onOpenShortPosition() {
        onPositionAction(.open(direction: PerpetualDirection.short.toGem()))
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
            try await observerService.subscribe(service.marketSubscription(perpetual: perpetual.toGem()))
        } catch {
            debugLog("Market data subscription failed: \(error)")
        }
    }

    func unsubscribeMarket() async {
        do {
            try await observerService.unsubscribe(service.marketSubscription(perpetual: perpetual.toGem()))
        } catch {
            debugLog("Market data unsubscribe failed: \(error)")
        }
    }

    func onPositionAction(_ kind: GemPerpetualPositionKind) {
        do {
            let positionAction = try service.positionAction(perpetual: perpetual.toGem(), asset: asset.toGem(), position: details.position, kind: kind)
            onPerpetualPosition?(positionAction)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    public func positionData(_ details: GemPerpetualDetails) -> PerpetualPositionData? {
        details.position.map { PerpetualPositionData(perpetual: perpetual, asset: asset, position: $0.toPrimitives()) }
    }

    func refreshStored() async {
        for failure in await service.refresh(assetId: asset.id.identifier) {
            debugLog("perpetual scene refresh error: \(failure.step) \(failure.message)")
        }
    }
}
