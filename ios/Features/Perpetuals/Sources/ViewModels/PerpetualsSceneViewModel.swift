// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemMarketsRefreshTrigger
import struct Gemstone.GemPerpetualBalanceHeader
import struct Gemstone.GemPerpetualMarketCounts
import enum Gemstone.GemPerpetualMarketSection
import struct Gemstone.GemPerpetualMarketSections
import struct Gemstone.GemPerpetualMarketSession
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemRecentActivityServiceProtocol
import func Gemstone.perpetualBalanceHeader
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class PerpetualsSceneViewModel {
    private let observerService: any PerpetualObservable
    private let service: any GemPerpetualServiceProtocol

    let wallet: Wallet

    let positionsQuery: ObservableQuery<PerpetualPositionsRequest>
    let perpetualsQuery: ObservableQuery<MappedRequest<PerpetualsRequest, GemPerpetualMarketSections>>
    let walletBalanceQuery: ObservableQuery<PerpetualWalletBalanceRequest>
    let recentModel: RecentAssetsModel

    var positions: [PerpetualPositionData] {
        positionsQuery.value
    }

    var sections: GemPerpetualMarketSections {
        perpetualsQuery.value
    }

    var balanceHeader: GemPerpetualBalanceHeader {
        perpetualBalanceHeader(balance: walletBalanceQuery.value?.balance.toGem(), walletType: wallet.type.toGem())
    }

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
    let onSelectAsset: ((Asset) -> Void)?
    let onSelectPortfolio: VoidAction

    public init(
        wallet: Wallet,
        service: any GemPerpetualServiceProtocol,
        observerService: any PerpetualObservable,
        recentAssetsService: any GemRecentActivityServiceProtocol,
        onSelectAmount: ((AmountInput) -> Void)? = nil,
        onSelectAsset: ((Asset) -> Void)? = nil,
        onSelectPortfolio: (() -> Void)? = nil,
    ) {
        self.wallet = wallet
        self.service = service
        self.observerService = observerService
        self.onSelectAmount = onSelectAmount
        self.onSelectAsset = onSelectAsset
        self.onSelectPortfolio = onSelectPortfolio
        positionsQuery = ObservableQuery(PerpetualPositionsRequest(walletId: wallet.id, searchQuery: ""), initialValue: [])
        perpetualsQuery = ObservableQuery(.marketSections(search: .empty), initialValue: GemPerpetualMarketSections(pinned: [], markets: []))
        walletBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceRequest(walletId: wallet.id, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: nil,
        )
        recentModel = RecentAssetsModel(walletId: wallet.id, types: [.perpetual], service: recentAssetsService)
    }

    var navigationTitle: String {
        Localized.Perpetuals.title
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.searchPerpetuals))
    }

    var pinImage: Image {
        Images.System.pin
    }

    var searchImage: Image {
        Images.System.search
    }

    var marketSectionList: [GemPerpetualMarketSection] {
        session.sections(counts: GemPerpetualMarketCounts(
            positions: UInt32(positions.count),
            pinned: UInt32(sections.pinned.count),
            markets: UInt32(sections.markets.count),
            recents: recentModel.hasAssets ? 1 : 0,
        ))
    }

    var headerViewModel: PerpetualsHeaderViewModel {
        PerpetualsHeaderViewModel(header: balanceHeader)
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

    func onSelectHeaderAction(type: GemHeaderButtonKind) {
        switch type {
        case .deposit:
            onSelectAmount?(AmountInput(type: .deposit, asset: balanceHeader.depositAsset.toPrimitives()))
        case .withdraw:
            onSelectAmount?(AmountInput(type: .withdraw, asset: balanceHeader.withdrawAsset.toPrimitives()))
        default:
            break
        }
    }

    func onPinPerpetual(_ perpetualData: PerpetualData) {
        Task {
            do {
                try await service.setPinned(!perpetualData.metadata.isPinned, perpetualId: perpetualData.perpetual.id)
            } catch {
                debugLog("PerpetualsSceneViewModel pin perpetual error: \(error)")
            }
        }
    }

    func onSearchQueryChange(_ _: String, _: String) {
        let query = session.searchQuery()
        perpetualsQuery.request = .marketSections(search: query)
        positionsQuery.request = PerpetualPositionsRequest(walletId: wallet.id, searchQuery: query)
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
