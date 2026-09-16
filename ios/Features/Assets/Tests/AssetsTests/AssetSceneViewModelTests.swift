// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
import BigInt
import class Gemstone.GemDeeplinkService
import protocol Gemstone.GemPriceAlertServiceProtocol
import struct Gemstone.GemSwapPairSuggestion
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct AssetSceneViewModelTests {
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
    func balanceRows() {
        let ethereum = AssetSceneViewModel.mock(
            .mock(
                asset: .mockEthereum(),
                balance: .mock(staked: BigInt(6_000_000_000_000_000_000), earn: BigInt(4_000_000_000_000_000_000)),
            ),
        )
        let rows = ethereum.balanceRows
        #expect(rows.count == 2)
        guard case let .staked(staked) = rows[1] else {
            Issue.record("Expected available and staked rows")
            return
        }
        #expect(ethereum.stakeBalanceText(staked) == "6 ETH")
        #expect(ethereum.balanceText(BigUInt(4_000_000_000_000_000_000)) == "4 ETH")
        #expect(AssetSceneViewModel.mock(.mock(asset: .mockEthereum(), metadata: .mock(isStakeEnabled: false))).balanceRows.isEmpty)
    }
}
