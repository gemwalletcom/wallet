// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import protocol Gemstone.GemPriceAlertServiceProtocol
import class Gemstone.StakeConfig
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import BigInt
import class Gemstone.GemTransactionFormatter
import Primitives
import PrimitivesTestKit
@testable import Store
import SwiftUI
import PreferencesTestKit
import Testing
import protocol Gemstone.GemSwapServiceProtocol
import struct Gemstone.GemSwapPairSuggestion

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
    func swapAssetTypeUsesTheAssetWhenCoreSuggestsNoReceiveAsset() {
        let asset = Asset.mockEthereumUSDT()
        let model = AssetSceneViewModel.mock(
            .mock(asset: asset, balance: .mock()),
            swapService: GemSwapServiceMock(assetPair: GemSwapPairSuggestion(payAssetId: asset.id.identifier, receiveAssetId: nil)),
        )

        #expect(model.swapAssetType == .swap(asset, nil))
    }

    @Test
    func swapAssetTypePaysWithTheChainAssetWhenCoreSuggestsAReceiveAsset() {
        let asset = Asset.mockEthereumUSDT()
        let model = AssetSceneViewModel.mock(
            .mock(asset: asset, balance: .zero),
            swapService: GemSwapServiceMock(
                assetPair: GemSwapPairSuggestion(payAssetId: asset.chain.assetId.identifier, receiveAssetId: asset.id.identifier),
            ),
        )

        #expect(model.swapAssetType == .swap(asset.chain.asset, asset))
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
        #expect(AssetSceneViewModel.mock(.mock(asset: .mockEthereum(), metadata: .mock(isStakeEnabled: true))).showProviderBalance(for: .stake) == true)
        #expect(AssetSceneViewModel.mock(.mock(asset: .mockEthereum(), balance: .mock(staked: BigInt(100)), metadata: .mock(isStakeEnabled: false))).showProviderBalance(for: .stake) == true)
        #expect(AssetSceneViewModel.mock(.mock(asset: .mockSolana(), balance: .mock(rewards: BigInt(100)), metadata: .mock(isStakeEnabled: false))).showProviderBalance(for: .stake) == true)
        #expect(AssetSceneViewModel.mock(.mock(asset: .mockEthereum(), metadata: .mock(isStakeEnabled: false))).showProviderBalance(for: .stake) == false)
        #expect(AssetSceneViewModel.mock(.mock(asset: .mock(), metadata: .mock(isStakeEnabled: true))).showProviderBalance(for: .stake) == false)
        #expect(AssetSceneViewModel.mock(.mock(balance: .mock(earn: BigInt(100)))).showProviderBalance(for: .earn) == true)
        #expect(AssetSceneViewModel.mock(.mock()).showProviderBalance(for: .earn) == false)
    }

    @Test
    func balanceTextWithSymbol() {
        let ethereum = AssetSceneViewModel.mock(.mock(
            asset: .mockEthereum(),
            balance: .mock(
                staked: BigInt(1_000_000_000_000_000_000),
                pending: BigInt(2_000_000_000_000_000_000),
                rewards: BigInt(3_000_000_000_000_000_000),
                earn: BigInt(4_000_000_000_000_000_000),
            ),
        ))
        #expect(ethereum.balanceTextWithSymbol(for: .stake) == "6 ETH")
        #expect(ethereum.balanceTextWithSymbol(for: .earn) == "4 ETH")

        let tron = AssetSceneViewModel.mock(.mock(
            asset: .mockTron(),
            balance: .mock(frozen: BigInt(1_000_000), locked: BigInt(2_000_000), staked: BigInt(9_000_000), rewards: BigInt(3_000_000)),
        ))
        #expect(tron.balanceTextWithSymbol(for: .stake) == "6 TRX")
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
    static func mock(
        _ assetData: AssetData = AssetData.mock(),
        swapService: any GemSwapServiceProtocol = GemSwapServiceMock(),
    ) -> AssetSceneViewModel {
        let model = AssetSceneViewModel(
            balanceService: GemBalanceServiceMock(),
            assetsService: GemAssetsServiceMock(),
            transactionsService: GemTransactionsServiceMock(),
            priceUpdater: .mock(),
            priceAlertService: GemPriceAlertServiceMock(),
            bannerService: GemBannerServiceMock(),
            swapService: swapService,
            stakeConfig: StakeConfig(),
            explorerService: GemExplorerServiceMock(),
            transactionFormatter: GemTransactionFormatter(),
            preferences: .mock(),
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
