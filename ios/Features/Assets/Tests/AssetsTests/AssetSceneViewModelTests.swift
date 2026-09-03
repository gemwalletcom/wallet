// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import class Gemstone.GemDeeplinkService
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import BigInt
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
@testable import Store
import SwiftUI
import PreferencesTestKit
import Testing
import protocol Gemstone.GemAssetDetailsServiceProtocol
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
            service: GemAssetDetailsServiceMock(assetPair: GemSwapPairSuggestion(payAssetId: asset.id.identifier, receiveAssetId: nil)),
        )

        #expect(model.swapAssetType == .swap(asset, nil))
    }

    @Test
    func swapAssetTypePaysWithTheChainAssetWhenCoreSuggestsAReceiveAsset() {
        let asset = Asset.mockEthereumUSDT()
        let model = AssetSceneViewModel.mock(
            .mock(asset: asset, balance: .zero),
            service: GemAssetDetailsServiceMock(
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
    func balanceRows() {
        let ethereum = AssetSceneViewModel.mock(
            .mock(
                asset: .mockEthereum(),
                balance: .mock(staked: BigInt(6_000_000_000_000_000_000), earn: BigInt(4_000_000_000_000_000_000)),
            )
        )
        let rows = ethereum.balanceRows
        #expect(rows.count == 3)
        guard case let .staked(staked) = rows[1], case let .earn(earn) = rows[2] else {
            Issue.record("Expected available, staked and earn rows")
            return
        }
        #expect(ethereum.stakeBalanceText(staked) == "6 ETH")
        #expect(ethereum.balanceText(earn) == "4 ETH")
        #expect(AssetSceneViewModel.mock(.mock(asset: .mockEthereum(), metadata: .mock(isStakeEnabled: false))).balanceRows.isEmpty)
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
        service: any GemAssetDetailsServiceProtocol = GemAssetDetailsServiceMock(),
    ) -> AssetSceneViewModel {
        let model = AssetSceneViewModel(
            service: service,
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
