// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAssetsServiceProtocol
import GemstonePrimitives
import protocol Gemstone.GemNftServiceProtocol
import protocol Gemstone.GemRecentActivityServiceProtocol
import GemstoneServices
import Components
import NFT
import Primitives
import PrimitivesComponents
import SwiftUI
import Transactions

@Observable
final class NavigationPresenter: Sendable {
    @MainActor private var _isPresentingAssetInput: SelectedAssetInput?
    @MainActor private var _isPresentingPayment: PaymentDestination?
    @MainActor private var _isPresentingPriceAlert: Asset?
    @MainActor private var _isPresentingSupport: Bool = false
    @MainActor private var _isPresentingWallets: Bool = false
    private let assetsService: any GemAssetsServiceProtocol
    private let nftService: any GemNftServiceProtocol
    private let recentActivity: any GemRecentActivityServiceProtocol

    init(
        assetsService: any GemAssetsServiceProtocol,
        nftService: any GemNftServiceProtocol,
        recentActivity: any GemRecentActivityServiceProtocol,
    ) {
        self.assetsService = assetsService
        self.nftService = nftService
        self.recentActivity = recentActivity
    }
}

@MainActor
extension NavigationPresenter {
    var isPresentingAssetInput: Binding<SelectedAssetInput?> {
        Binding(get: { self._isPresentingAssetInput }, set: { self._isPresentingAssetInput = $0 })
    }

    var isPresentingPayment: Binding<PaymentDestination?> {
        Binding(get: { self._isPresentingPayment }, set: { self._isPresentingPayment = $0 })
    }

    var isPresentingPriceAlert: Binding<Asset?> {
        Binding(get: { self._isPresentingPriceAlert }, set: { self._isPresentingPriceAlert = $0 })
    }

    var isPresentingSupport: Binding<Bool> {
        Binding(get: { self._isPresentingSupport }, set: { self._isPresentingSupport = $0 })
    }

    var isPresentingWallets: Binding<Bool> {
        Binding(get: { self._isPresentingWallets }, set: { self._isPresentingWallets = $0 })
    }

    func presentAssetInput(type: SelectedAssetType, for asset: Asset, wallet: Wallet) throws {
        let account = try wallet.account(for: asset.chain)
        isPresentingAssetInput.wrappedValue = SelectedAssetInput(
            type: type,
            assetData: .with(asset: asset, account: account),
        )
    }

    func presentSwap(
        from fromAssetId: AssetId,
        to toAssetId: AssetId?,
        wallet: Wallet,
    ) async throws {
        let fromAsset = try await assetsService.ensureAsset(for: fromAssetId)
        let toAsset: Asset? = if let toAssetId {
            try await assetsService.ensureAsset(for: toAssetId)
        } else {
            nil
        }
        try presentAssetInput(type: .swap(fromAsset, toAsset), for: fromAsset, wallet: wallet)
    }

    func handleTransactionHeaderAction(
        _ action: TransactionHeaderAction,
        wallet: Wallet,
        navigationState: NavigationStateManager,
        nftDestination: NavigationPathState,
    ) async throws {
        switch action {
        case let .asset(assetId), let .perpetual(assetId):
            guard let asset = try await assetsService.openWalletAsset(wallet: wallet, assetId: assetId) else {
                return
            }
            navigationState.openAsset(asset)
        case let .swap(fromAssetId, toAssetId):
            try await presentSwap(
                from: fromAssetId,
                to: toAssetId,
                wallet: wallet,
            )
        case let .nft(assetId):
            let assetData = try NFTAssetData(await nftService.ensureAsset(assetId: assetId.identifier))
            nftDestination.append(Scenes.Collectible(assetData: assetData))
        }
    }

    func recordRecent(input: SelectedAssetInput) {
        guard let action = input.type.action else { return }
        Task { try? await recentActivity.addRecent(action: action, asset: input.asset.map()) }
    }

    func completeSwap(fromAsset: Asset, navigationState: NavigationStateManager) async throws {
        let asset = try await assetsService.ensureAsset(for: fromAsset.id)
        switch navigationState.selectedTab {
        case .wallet:
            navigationState.wallet.setPath([Scenes.Asset(asset: asset)])
        case .activity:
            navigationState.wallet.setPath([Scenes.Asset(asset: asset)])
            navigationState.selectedTab = .wallet
        case .settings:
            break
        }
        isPresentingAssetInput.wrappedValue = nil
    }
}
