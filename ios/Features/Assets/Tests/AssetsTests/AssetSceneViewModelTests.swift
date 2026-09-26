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
        let asset = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let model = AssetSceneViewModel.mock(
            .mock(asset: asset, balance: .mock()),
            service: GemAssetDetailsServiceMock(assetPair: GemSwapPairSuggestion(payAssetId: asset.id.identifier, receiveAssetId: nil)),
        )

        #expect(model.swapAssetType == .swap(asset.id, nil))
    }

    @Test
    func swapAssetTypePaysWithTheChainAssetWhenCoreSuggestsAReceiveAsset() {
        let asset = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
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
        let asset = Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)
        let model = AssetSceneViewModel.mock(.mock(asset: asset, balance: .mock()))

        #expect(model.assetHeader(model.details).title == GemFormattedNumber.mock(value: 0, unit: .symbol(symbol: asset.symbol), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest)
            .text())
    }

    @Test
    func balanceRowsShowCoreValues() {
        let model = AssetSceneViewModel.mock(.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)))
        let apr = GemFormattedNumber.mock(value: 3.24, unit: .percent, display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest)

        #expect(model.balanceListItem(for: GemAssetBalanceRow(row: .staked(value: 0), value: .apr(apr: apr))).subtitle == Localized.Stake.apr("3.24%"))
        #expect(model.balanceListItem(for: GemAssetBalanceRow(row: .staked(value: 0), value: .apr(apr: nil))).subtitle == Localized.Stake.apr(""))
        #expect(model.balanceListItem(for: GemAssetBalanceRow(
            row: .pendingUnconfirmed(value: 1),
            value: .amount(amount: .mock(value: 1, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest)),
        )).infoAction != nil)
    }

    @Test
    func detailRowsKeepTheirScreenshotIdentifiers() {
        let model = AssetSceneViewModel.mock(.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)))
        let apr = GemAssetBalanceRow(row: .staked(value: 0), value: .apr(apr: nil))

        #expect(model.accessibilityIdentifier(.balance(row: apr, action: .stake)) == "stake")
        #expect(model.accessibilityIdentifier(.balance(row: GemAssetBalanceRow(row: .earn(value: 0), value: .apr(apr: nil)), action: .earn)) == "earn")
        #expect(model.accessibilityIdentifier(.row(row: .quote(title: .price, value: nil, change: nil), action: .price)) == "price")
        #expect(model.accessibilityIdentifier(.row(row: .network(title: .network, chain: Chain.ethereum.rawValue, name: "Ethereum"), action: .network)) == nil)
    }
}
