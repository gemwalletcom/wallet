// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemAssetsServiceProtocol
import enum Gemstone.GemErrorText
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import SwiftUI
import Transfer
import Wallet

@Observable
final class NavigationPresenter: Sendable {
    @MainActor private var _isPresentingAddressDetails: ChainAddress?
    @MainActor private var _isPresentingAssetInput: SelectedAssetInput?
    @MainActor private var _isPresentingPayment: PaymentDestination?
    @MainActor private var _isPresentingPriceAlert: Asset?
    @MainActor private var _isPresentingSupport: Bool = false
    @MainActor private var _isPresentingWallets: Bool = false
    @MainActor private var _isPresentingWalletSheet: WalletSheetType?
    private let assetsService: any GemAssetsServiceProtocol

    init(assetsService: any GemAssetsServiceProtocol) {
        self.assetsService = assetsService
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

    var isPresentingWalletSheet: Binding<WalletSheetType?> {
        Binding(get: { self._isPresentingWalletSheet }, set: { self._isPresentingWalletSheet = $0 })
    }

    func reset() {
        _isPresentingAddressDetails = nil
        _isPresentingAssetInput = nil
        _isPresentingPayment = nil
        _isPresentingPriceAlert = nil
        _isPresentingSupport = false
        _isPresentingWallets = false
        _isPresentingWalletSheet = nil
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
