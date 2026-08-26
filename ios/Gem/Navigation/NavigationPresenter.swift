// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import protocol Gemstone.GemNftServiceProtocol
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
    @MainActor private var _isPresentingPriceAlert: SetPriceAlertInput?
    @MainActor private var _isPresentingSupport: Bool = false
    @MainActor private var _isPresentingWallets: Bool = false

    init() {}
}

@MainActor
extension NavigationPresenter {
    var isPresentingAssetInput: Binding<SelectedAssetInput?> {
        Binding(get: { self._isPresentingAssetInput }, set: { self._isPresentingAssetInput = $0 })
    }

    var isPresentingPayment: Binding<PaymentDestination?> {
        Binding(get: { self._isPresentingPayment }, set: { self._isPresentingPayment = $0 })
    }

    var isPresentingPriceAlert: Binding<SetPriceAlertInput?> {
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
        assetsService: AssetsService,
    ) async throws {
        let fromAsset = try await assetsService.getOrFetchAsset(for: fromAssetId)
        let toAsset: Asset? = if let toAssetId {
            try await assetsService.getOrFetchAsset(for: toAssetId)
        } else {
            nil
        }
        try presentAssetInput(type: .swap(fromAsset, toAsset), for: fromAsset, wallet: wallet)
    }

    func handleTransactionHeaderAction(
        _ action: TransactionHeaderAction,
        wallet: Wallet,
        navigationState: NavigationStateManager,
        assetsService: AssetsService,
        nftService: any GemNftServiceProtocol,
        nftDestination: NavigationPathState,
    ) async throws {
        switch action {
        case let .asset(assetId), let .perpetual(assetId):
            guard wallet.accounts.contains(where: { $0.chain == assetId.chain }) else {
                return
            }
            let asset = try await assetsService.getOrFetchAsset(for: assetId)
            navigationState.openAsset(asset)
        case let .swap(fromAssetId, toAssetId):
            try await presentSwap(
                from: fromAssetId,
                to: toAssetId,
                wallet: wallet,
                assetsService: assetsService,
            )
        case let .nft(assetId):
            let assetData = try NFTAssetData(await nftService.getOrFetchAsset(assetId: assetId.identifier))
            nftDestination.append(Scenes.Collectible(assetData: assetData))
        }
    }
}
