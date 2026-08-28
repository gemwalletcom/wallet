// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstoneServices
import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

@Observable
@MainActor
final class PerpetualsSceneViewModel {

    private let observerService: any PerpetualObservable
    let perpetualService: any GemPerpetualServiceProtocol
    let recentActivityStore: RecentActivityStore

    let wallet: Wallet

    let positionsQuery: ObservableQuery<PerpetualPositionsRequest>
    let perpetualsQuery: ObservableQuery<PerpetualsRequest>
    let walletBalanceQuery: ObservableQuery<PerpetualWalletBalanceRequest>
    let recentModel: RecentAssetsModel

    var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    var perpetuals: [PerpetualData] {
        perpetualsQuery.value
    }

    var walletBalance: WalletBalance {
        walletBalanceQuery.value
    }

    var isSearchPresented: Bool = false
    var searchQuery: String = .empty
    var isSearching: Bool = false

    let onSelectAssetType: ((SelectAssetType) -> Void)?
    let onSelectAsset: ((Asset) -> Void)?
    let onSelectPortfolio: VoidAction

    init(
        wallet: Wallet,
        perpetualService: any GemPerpetualServiceProtocol,
        observerService: any PerpetualObservable,
        recentActivityStore: RecentActivityStore,
        onSelectAssetType: ((SelectAssetType) -> Void)? = nil,
        onSelectAsset: ((Asset) -> Void)? = nil,
        onSelectPortfolio: (() -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.perpetualService = perpetualService
        self.observerService = observerService
        self.recentActivityStore = recentActivityStore
        self.onSelectAssetType = onSelectAssetType
        self.onSelectAsset = onSelectAsset
        self.onSelectPortfolio = onSelectPortfolio
        positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: wallet.id, searchQuery: ""), initialValue: [])
        perpetualsQuery = ObservableQuery(PerpetualsRequest(searchQuery: ""), initialValue: [])
        walletBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceRequest(walletId: wallet.id, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: .zero,
        )
        recentModel = RecentAssetsModel(walletId: wallet.id, types: [.perpetual], recentActivityStore: recentActivityStore)
    }

    var navigationTitle: String {
        Localized.Perpetuals.title
    }

    var positionsSectionTitle: String {
        Localized.Perpetual.positions
    }

    var marketsSectionTitle: String {
        Localized.Perpetuals.markets
    }

    var pinnedSectionTitle: String {
        Localized.Common.pinned
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .search(type: .perpetuals))
    }

    var pinImage: Image {
        Images.System.pin
    }

    var searchImage: Image {
        Images.System.search
    }

    var showPositions: Bool {
        positions.isNotEmpty
    }

    var showPinned: Bool {
        sections.pinned.isNotEmpty
    }

    var showMarkets: Bool {
        sections.markets.isNotEmpty
    }

    var showRecents: Bool {
        isSearching && searchQuery.isEmpty && recentModel.hasAssets
    }

    var showSearchEmptyState: Bool {
        isSearching && !showPositions && !showPinned && !showMarkets
    }

    var sections: PerpetualsSections {
        .from(perpetuals)
    }

    var headerViewModel: PerpetualsHeaderViewModel {
        PerpetualsHeaderViewModel(
            walletType: wallet.type,
            balance: walletBalance,
        )
    }
}

// MARK: - Business Logic

extension PerpetualsSceneViewModel {
    func load() async {
        async let updateObserver: PerpetualAccountMode? = observerService.update(for: wallet)
        async let refreshMarkets: () = updateMarkets()
        _ = await (updateObserver, refreshMarkets)
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

    func updateMarkets() async {
        do {
            try await perpetualService.updateMarkets()
        } catch {
            debugLog("Failed to update markets: \(error)")
        }
    }

    func onSelectHeaderAction(type: HeaderButtonType) {
        switch type {
        case .deposit:
            onSelectAssetType?(.deposit)
        case .withdraw:
            onSelectAssetType?(.withdraw)
        default:
            break
        }
    }

    func onPinPerpetual(_ perpetualData: PerpetualData) {
        Task {
            do {
                try await perpetualService.setPinned(!perpetualData.metadata.isPinned, perpetualId: perpetualData.perpetual.id)
            } catch {
                debugLog("PerpetualsSceneViewModel pin perpetual error: \(error)")
            }
        }
    }

    func onSearchQueryChange(_ _: String, _ newValue: String) {
        let trimmed = newValue.trimmingCharacters(in: .whitespacesAndNewlines)
        perpetualsQuery.request = PerpetualsRequest(searchQuery: trimmed)
        positionsQuery.request = PerpetualPositionsRequest(walletId: wallet.id, searchQuery: trimmed)
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
        do {
            try recentActivityStore.add(
                RecentActivityData(type: .perpetual, assetId: asset.id, toAssetId: nil),
                walletId: wallet.id,
            )
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }

    func onSelectRecent(asset: Asset) {
        onSelectAsset?(asset)
        recentModel.dismiss()
    }

    func onSelectBalance() {
        onSelectPortfolio?()
    }
}
