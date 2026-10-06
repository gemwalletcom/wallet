// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemAssetsServiceProtocol
import enum Gemstone.GemErrorText
import protocol Gemstone.GemNavigationServiceProtocol
import enum Gemstone.GemTransactionHeaderAction
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import SwiftUI
import Transfer

@Observable
final class NavigationPresenter: Sendable {
    @MainActor private var _isPresentingAddressDetails: ChainAddress?
    @MainActor private var _isPresentingAssetInput: SelectedAssetInput?
    @MainActor private var _isPresentingPayment: PaymentDestination?
    @MainActor private var _isPresentingPriceAlert: Asset?
    @MainActor private var _isPresentingSupport: Bool = false
    @MainActor private var _isPresentingWallets: Bool = false
    private let assetsService: any GemAssetsServiceProtocol
    private let navigationService: any GemNavigationServiceProtocol

    init(
        assetsService: any GemAssetsServiceProtocol,
        navigationService: any GemNavigationServiceProtocol,
    ) {
        self.assetsService = assetsService
        self.navigationService = navigationService
    }
}

@MainActor
extension NavigationPresenter {
    var isPresentingAddressDetails: Binding<ChainAddress?> {
        Binding(get: { self._isPresentingAddressDetails }, set: { self._isPresentingAddressDetails = $0 })
    }

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
        guard let account = try? wallet.account(for: asset.chain) else {
            throw GemErrorText.noAccountForChain
        }
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
        let fromAsset = try await assetsService.ensureAsset(assetId: fromAssetId).toPrimitives()
        if let toAssetId {
            _ = try await assetsService.ensureAsset(assetId: toAssetId)
        }
        try presentAssetInput(type: .swap(fromAssetId, toAssetId), for: fromAsset, wallet: wallet)
    }

    func openTransactionHeaderAction(
        _ action: GemTransactionHeaderAction,
        wallet: Wallet,
        navigationState: NavigationStateManager,
        nftDestination: NavigationPathState,
    ) async throws {
        switch action {
        case let .asset(assetId), let .perpetual(assetId):
            try await navigationState.openAsset(target: navigationService.openAsset(assetId: assetId))
        case let .swap(fromAssetId, toAssetId):
            try await presentSwap(
                from: fromAssetId,
                to: toAssetId,
                wallet: wallet,
            )
        case let .nft(assetId):
            try nftDestination.append(Scenes.Collectible.assetId(NFTAssetId.from(id: assetId)))
        }
    }

    func completeSwap(fromAssetId: AssetId, navigationState: NavigationStateManager) async throws {
        let asset = try await assetsService.ensureAsset(assetId: fromAssetId).toPrimitives()
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
