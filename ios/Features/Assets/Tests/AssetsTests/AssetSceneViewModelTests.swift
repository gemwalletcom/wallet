// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
import struct Gemstone.GemAssetBalanceRow
import class Gemstone.GemDeeplinkService
import struct Gemstone.GemFormattedNumber
import protocol Gemstone.GemPriceAlertServiceProtocol
import struct Gemstone.GemSwapPairSuggestion
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Localization
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

        #expect(model.swapAssetType == .swap(asset.id, nil))
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

        #expect(model.swapAssetType == .swap(asset.chain.assetId, asset.id))
    }

    @Test
    func theHeaderBalanceIsTheOneCoreFormatted() {
        let asset = Asset.mockEthereum()
        let model = AssetSceneViewModel.mock(.mock(asset: asset, balance: .mock()))

        #expect(model.assetHeaderModel(model.details).title == GemFormattedNumber.mock(value: 0, unit: .symbol(symbol: asset.symbol)).text())
    }

    @Test
    func balanceRowsShowCoreValues() {
        let model = AssetSceneViewModel.mock(.mock(asset: .mockEthereum()))
        let apr = GemFormattedNumber.mock(value: 3.24, unit: .percent, notation: .plain)

        #expect(model.balanceListItem(for: GemAssetBalanceRow(row: .staked(value: 0), value: .apr(apr: apr))).subtitle == Localized.Stake.apr("3.24%"))
        #expect(model.balanceListItem(for: GemAssetBalanceRow(row: .staked(value: 0), value: .apr(apr: nil))).subtitle == Localized.Stake.apr(""))
        #expect(model.balanceListItem(for: GemAssetBalanceRow(row: .pendingUnconfirmed(value: 1), value: .amount(amount: .mock()))).infoAction != nil)
    }
}
