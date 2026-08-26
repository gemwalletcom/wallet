// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import BigInt
import Primitives
import PrimitivesTestKit
@testable import Store
import SwiftUI
import Testing

@MainActor
struct AssetSceneViewModelTests {
    @Test
    func showManageToken() {
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(isBalanceEnabled: true))).showManageToken == false)
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(isBalanceEnabled: false))).showManageToken == true)
    }

    @Test
    func showStatus() {
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(rankScore: 42))).showStatus == false)
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(rankScore: 10))).showStatus == true)
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(rankScore: 3))).showStatus == false)
    }

    @Test
    func swapAssetTypeNative() {
        let asset = Asset.mock(type: .native)
        let model = AssetSceneViewModel.mock(.mock(asset: asset, balance: .zero))

        #expect(model.swapAssetType == .swap(asset, nil))
    }

    @Test
    func swapAssetTypeTokenWithZeroBalance() {
        let asset = Asset.mockEthereumUSDT()
        let model = AssetSceneViewModel.mock(.mock(asset: asset, balance: .zero))

        #expect(model.swapAssetType == .swap(asset.chain.asset, asset))
    }

    @Test
    func swapAssetTypeTokenWithBalance() {
        let asset = Asset.mockEthereumUSDT()
        let model = AssetSceneViewModel.mock(.mock(asset: asset, balance: .mock()))

        #expect(model.swapAssetType == .swap(asset, nil))
    }

    @Test
    func swapAssetTypeDoesNotSelectHiddenNativeAsset() {
        let asset = Asset.mockTempoUSDC()
        let model = AssetSceneViewModel.mock(.mock(asset: asset, balance: .zero))

        #expect(model.swapAssetType == .swap(asset, nil))
    }

    @Test
    func networkDestinationUsesVisibleNativeAsset() {
        let asset = Asset.mockEthereumUSDT()
        let model = AssetSceneViewModel.mock(.mock(asset: asset))

        #expect(model.networkDestination == .asset(asset.chain.asset))
    }

    @Test
    func networkDestinationUsesAssetListWithoutNativeAsset() {
        let asset = Asset.mockTempoUSDC()
        let model = AssetSceneViewModel.mock(.mock(asset: asset))

        #expect(model.networkDestination == .assets(.tempo))
    }

    @Test
    func showProviderBalance() {
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(isStakeEnabled: true))).showProviderBalance(for: .stake) == true)
        #expect(AssetSceneViewModel.mock(.mock(balance: .mock(staked: BigInt(100)), metadata: .mock(isStakeEnabled: false))).showProviderBalance(for: .stake) == true)
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(isStakeEnabled: false))).showProviderBalance(for: .stake) == false)
        #expect(AssetSceneViewModel.mock(.mock(balance: .mock(earn: BigInt(100)))).showProviderBalance(for: .earn) == true)
        #expect(AssetSceneViewModel.mock(.mock()).showProviderBalance(for: .earn) == false)
    }

    @Test
    func showEarnButton() {
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(isEarnEnabled: true))).showEarnButton == true)
        #expect(AssetSceneViewModel.mock(.mock(metadata: .mock(isEarnEnabled: false))).showEarnButton == false)
        #expect(AssetSceneViewModel.mock(.mock(balance: .mock(earn: BigInt(100)), metadata: .mock(isEarnEnabled: true))).showEarnButton == false)
    }

    @Test
    func balanceTitle() {
        let model = AssetSceneViewModel.mock()
        #expect(model.balanceTitle(for: .stake).isEmpty == false)
        #expect(model.balanceTitle(for: .earn).isEmpty == false)
    }
}

// MARK: - Mock Extensions

extension AssetSceneViewModel {
    static func mock(_ assetData: AssetData = AssetData.mock()) -> AssetSceneViewModel {
        let model = AssetSceneViewModel(
            assetsEnabler: .mock(),
            balanceService: GemBalanceServiceMock(),
            assetsService: GemAssetsServiceMock(),
            transactionsService: GemTransactionsServiceMock(),
            priceUpdater: .mock(),
            priceAlertService: .mock(),
            bannerService: GemBannerServiceMock(),
            explorerService: GemExplorerServiceMock(),
            input: AssetSceneInput(
                wallet: .mock(),
                asset: assetData.asset,
            ),
            isPresentingSelectedAssetInput: .constant(.none),
        )
        model.assetQuery.value = ChainAssetData(
            assetData: assetData,
            feeAssetData: AssetData.with(asset: assetData.asset.chain.asset),
        )
        return model
    }
}
