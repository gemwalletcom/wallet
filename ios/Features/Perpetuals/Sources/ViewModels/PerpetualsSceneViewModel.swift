// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Foundation
import enum Gemstone.GemHeaderButtonAction
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemMarketsRefreshTrigger
import struct Gemstone.GemPerpetualMarketSections
import struct Gemstone.GemPerpetualMarketSession
import struct Gemstone.GemPerpetualMarketView
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemRecentActivityServiceProtocol
import struct Gemstone.GemValueHeader
import func Gemstone.perpetualBalanceHeader
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class PerpetualsSceneViewModel {
    private let observerService: any PerpetualObservable
    private let service: any GemPerpetualServiceProtocol

    let wallet: Wallet

    let positionsQuery: ObservableQuery<PerpetualPositionsQuery>
    let perpetualsQuery: ObservableQuery<MappedQuery<PerpetualsQuery, GemPerpetualMarketSections>>
    let walletBalanceQuery: ObservableQuery<PerpetualWalletBalanceQuery>
    let recentModel: RecentAssetsViewModel

    var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    var sections: GemPerpetualMarketSections {
        perpetualsQuery.value
    }

    var balanceHeader: GemValueHeader {
        perpetualBalanceHeader(balance: walletBalanceQuery.value?.balance.toGem(), walletType: wallet.type.toGem())
    }

    var isPresentingInfoSheet: InfoSheetModel?
    var isSearchPresented: Bool = false
    private var session = GemPerpetualMarketSession(query: .empty, isSearching: false)

    var searchQuery: String {
        get { session.query }
        set { session = session.onQueryChanged(query: newValue) }
    }

    var isSearching: Bool {
        get { session.isSearching }
        set { session = session.onSearchingChanged(isSearching: newValue) }
    }

    let onSelectAmount: ((AmountInput) -> Void)?
    let onSelectAssetType: ((SelectAssetType) -> Void)?
    let onSelectAsset: ((Asset) -> Void)?
    let onSelectPortfolio: VoidAction

    public init(
        wallet: Wallet,
        service: any GemPerpetualServiceProtocol,
        observerService: any PerpetualObservable,
        recentAssetsService: any GemRecentActivityServiceProtocol,
        onSelectAmount: ((AmountInput) -> Void)? = nil,
        onSelectAssetType: ((SelectAssetType) -> Void)? = nil,
        onSelectAsset: ((Asset) -> Void)? = nil,
        onSelectPortfolio: (() -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.service = service
        self.observerService = observerService
        self.onSelectAmount = onSelectAmount
        self.onSelectAssetType = onSelectAssetType
        self.onSelectAsset = onSelectAsset
        self.onSelectPortfolio = onSelectPortfolio
        positionsQuery = ObservableQuery(PerpetualPositionsQuery(walletId: wallet.id, searchQuery: ""), initialValue: [])
        perpetualsQuery = ObservableQuery(.marketSections(search: .empty), initialValue: GemPerpetualMarketSections(pinned: [], markets: []))
        walletBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceQuery(walletId: wallet.id, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: nil,
        )
        recentModel = RecentAssetsViewModel(walletId: wallet.id, types: [.perpetual], service: recentAssetsService)
    }

    var navigationTitle: String {
        Localized.Perpetuals.title
    }

    var pinImage: Image {
        Images.System.pin
    }

    var searchImage: Image {
        Images.System.search
    }

    var marketView: GemPerpetualMarketView {
        session.view(
            positionIds: positions.map(\.position.id),
            pinnedIds: sections.pinned.map(\.data.perpetual.id),
            marketIds: sections.markets.map(\.data.perpetual.id),
            recentAssetIds: recentModel.assets.map(\.asset.id),
        )
    }

    var header: ValueHeader {
        balanceHeader.valueHeader
    }
}

// MARK: - Business Logic

extension PerpetualsSceneViewModel {
    func load(source: RefreshSource = .timer) async {
        for failure in await service.refresh(trigger: source.marketsRefreshTrigger) {
            debugLog("Perpetuals refresh failed at \(failure.step): \(failure.message)")
        }
    }

    func onAppear() async {
        do {
            try await observerService.subscribe(.marketPrices)
        } catch {
            debugLog("Market prices subscribe failed: \(error)")
        }
    }

    func onDisappear() async {
        do {
            try await observerService.unsubscribe(.marketPrices)
        } catch {
            debugLog("Market prices unsubscribe failed: \(error)")
        }
    }

    func onSelectHeaderAction(_ action: GemHeaderButtonAction) {
        switch action {
        case .deposit:
            Task { await onSelectDeposit() }
        case let .withdraw(asset):
            onSelectAmount?(AmountInput(type: .withdraw, asset: asset.toPrimitives()))
        case .send, .receive, .buy, .swap, .sendCollectible, .collectibleMenu:
            break
        }
    }

    func onSelectDeposit() async {
        guard service.isAvailable() else {
            isPresentingInfoSheet = InfoSheetModel(sheet: GemInfoTopic.regionUnavailable.infoSheet)
            return
        }
        do {
            switch try await service.depositTarget() {
            case .selectAsset: onSelectAssetType?(.deposit)
            case let .amount(asset): onSelectAmount?(AmountInput(type: .deposit, asset: asset.toPrimitives()))
            }
        } catch {
            debugLog("PerpetualsSceneViewModel deposit target error: \(error)")
        }
    }

    func onPinPerpetual(_ perpetualData: PerpetualData) {
        Task {
            do {
                try await service.setPinned(perpetualId: perpetualData.perpetual.id, pinned: !perpetualData.metadata.isPinned)
            } catch {
                debugLog("PerpetualsSceneViewModel pin perpetual error: \(error)")
            }
        }
    }

    func onSearchQueryChange(_ _: String, _: String) {
        let query = session.searchQuery()
        perpetualsQuery.request = .marketSections(search: query)
        positionsQuery.request = PerpetualPositionsQuery(walletId: wallet.id, searchQuery: query)
    }

    func onSearchPresentedChange(_ _: Bool, _ isPresented: Bool) {
        if !isPresented {
            searchQuery = .empty
        }
    }

    func onSelectSearchButton() {
        isSearchPresented = true
    }

    func onSelectPerpetual(asset: Asset) {
        onSelectAsset?(asset)
        recentModel.add(action: .open, asset: asset)
    }

    func onSelectRecent(asset: Asset) {
        onSelectAsset?(asset)
        recentModel.dismiss()
    }

    func onSelectBalance() {
        onSelectPortfolio?()
    }
}

private extension RefreshSource {
    var marketsRefreshTrigger: GemMarketsRefreshTrigger {
        switch self {
        case .timer: .scheduled
        case .user: .userRequested
        }
    }
}
