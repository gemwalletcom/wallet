// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemLocalizedText
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct NetworkFeeSceneViewModelTests {
    @Test
    func additionalFeesKeepNetworkFeeTotal() {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9),
            feeAmount: 1_495_940,
            additionalFees: [(.tokenAccountCreation, 1_488_440), (.tokenAccountCreation, 1000)],
        )

        #expect(model.value == "0.001495 SOL")
        #expect(model.feeItems.map(\.title) == ["Account Activation Fee", "Account Activation Fee"])
        #expect(model.feeItems.first?.subtitle == "0.001488 SOL")
        #expect(NetworkFeeSceneViewModel.mock().feeItems.isEmpty)
    }

    @Test
    func showFeeRatesSelector() {
        #expect(NetworkFeeSceneViewModel.mock(feeRates: .mock(
            rows: [.mock(kind: .priority(priority: .normal), isSelected: true)],
            showsOptions: false,
            unitType: .gwei,
            unitDecimals: 9,
            selected: .regular(gasPrice: 1),
            normal: .regular(gasPrice: 1),
        )).showFeeRates == false)
        #expect(NetworkFeeSceneViewModel.mock(feeRates: .mock(
            rows: [.mock(kind: .priority(priority: .normal), isSelected: true), .mock(kind: .priority(priority: .fast))],
            showsOptions: true,
            unitType: .gwei,
            unitDecimals: 9,
            selected: .regular(gasPrice: 1),
            normal: .regular(gasPrice: 1),
        )).showFeeRates)
    }

    @Test
    func showFeeAssetsOnlyWhenAlternativeAssetIsSelectable() {
        let pathUSD = FeeAssetItem.mock(asset: .mock(id: .mock(chain: .tempo, tokenId: "0x20C0000000000000000000000000000000000000"), name: "pathUSD", symbol: "pathUSD", decimals: 6, type: .tip20))
        let usdc = FeeAssetItem.mock(asset: .mock(id: .mock(chain: .tempo, tokenId: "0x20C000000000000000000000b9537d11c60E8b50"), name: "Bridged USDC", symbol: "USDC.e", decimals: 6, type: .tip20))
        let onSelect: @MainActor (AssetId) -> Void = { _ in }
        let selectable = NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD, usdc], showsFeeAssets: true, onSelectFeeAsset: onSelect)

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD], onSelectFeeAsset: onSelect).showFeeAssets == false)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD, usdc], showsFeeAssets: true).showFeeAssets == false)
        #expect(selectable.showFeeAssets)
    }

    @Test
    func selectFeeAssetForwardsAssetIdToOwner() async {
        let pathUSD = FeeAssetItem.mock(asset: .mock(id: .mock(chain: .tempo, tokenId: "0x20C0000000000000000000000000000000000000"), name: "pathUSD", symbol: "pathUSD", decimals: 6, type: .tip20))
        let usdc = FeeAssetItem.mock(asset: .mock(id: .mock(chain: .tempo, tokenId: "0x20C000000000000000000000b9537d11c60E8b50"), name: "Bridged USDC", symbol: "USDC.e", decimals: 6, type: .tip20))

        await confirmation { selected in
            let model = NetworkFeeSceneViewModel.mock(
                feeAsset: pathUSD.asset,
                feeAssets: [pathUSD, usdc],
                onSelectFeeAsset: {
                    #expect($0 == usdc.asset.id)
                    selected()
                },
            )
            model.selectFeeAsset(usdc)
        }
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9),
            feeRates: .mock(
                rows: [.mock(kind: .priority(priority: .normal), fee: 5000, isSelected: true)],
                showsOptions: false,
                unitType: .native,
                unitDecimals: 9,
                selected: .regular(gasPrice: 5000),
                normal: .regular(gasPrice: 5000),
            ),
            feeAssetPrice: .mock(price: 150.0),
            feeAmount: BigInt(5000),
        )
        let row = try #require(model.feeRateRows.first)

        #expect(model.rowItem(for: row).subtitleExtra != nil)
    }

    @Test
    func fiatValueForNonNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeRates: .mock(
                rows: [.mock(kind: .priority(priority: .normal), fee: 21_000_000_000_000, isSelected: true)],
                showsOptions: false,
                unitType: .gwei,
                unitDecimals: 9,
                selected: .regular(gasPrice: 1),
                normal: .regular(gasPrice: 1),
            ),
            feeAssetPrice: .mock(price: 3000.0),
            feeAmount: BigInt(21_000_000_000_000),
        )
        let row = try #require(model.feeRateRows.first)

        #expect(model.rowItem(for: row).subtitleExtra != nil)
    }

    @Test
    func fiatValueNilWithoutPriceData() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9),
            feeRates: .mock(
                rows: [.mock(kind: .priority(priority: .normal), fee: 5000, isSelected: true)],
                showsOptions: false,
                unitType: .native,
                unitDecimals: 9,
                selected: .regular(gasPrice: 5000),
                normal: .regular(gasPrice: 5000),
            ),
            feeAmount: BigInt(5000),
        )
        let row = try #require(model.feeRateRows.first)

        #expect(model.rowItem(for: row).subtitleExtra == nil)
    }

    @Test
    func selectForwardsSelectionToOwner() async {
        await confirmation { selected in
            NetworkFeeSceneViewModel.mock(feeAsset: .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9), onSelect: {
                #expect($0 == .priority(priority: .fast))
                selected()
            })
            .select(.priority(priority: .fast))
        }
    }

    @Test
    func customFeeConfirmForwardsSelectionToOwner() async {
        await confirmation { selected in
            let custom = NetworkFeeSceneViewModel.mock(
                feeAsset: .mock(),
                feeRates: .mock(
                    rows: [.mock(kind: .priority(priority: .normal), fee: 1000, isSelected: true)],
                    showsOptions: false,
                    unitType: .satVb,
                    unitDecimals: 1,
                    selected: .regular(gasPrice: 20),
                    normal: .regular(gasPrice: 20),
                ),
                feeAmount: 1000,
                onSelect: {
                    #expect($0 == .custom(baseFee: nil, rate: 40))
                    selected()
                },
            ).customFeeModel()!
            custom.input = "4"
            custom.confirm()
        }
    }

    @Test
    func customRowDrawsTheCoreRate() {
        let customRate = GemFormattedNumber.mock(value: 20, unit: .plain, display: .number(precision: .fraction(min: 0, max: 1)), notation: .plain, tone: .plain, rounding: .toNearest)
        let row = { (value: GemLocalizedText?) in
            GemFeeRateRow.mock(kind: .custom, title: .customFee, value: value, isSelected: value != nil)
        }
        let model = NetworkFeeSceneViewModel.mock(feeAsset: .mock())

        #expect(model.rowItem(for: row(.feeRate(rate: customRate, unit: .satVb))).subtitle == "20 sat/vB")
        #expect(model.rowItem(for: row(nil)).subtitle == nil)
        #expect(model.rowItem(for: row(nil)).title == Localized.FeeRate.custom)
    }

    @Test
    func valueUsesFeeAssetForHyperCorePerpetualFee() {
        let feeAmount = BigInt(12_345_678)
        let feeAsset = Asset.mock(id: .mock(chain: .hyperCore, tokenId: "perpetual::USDC"), name: "USDC", symbol: "USDC", decimals: 6, type: .perpetual)
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: feeAsset,
            feeAmount: feeAmount,
        )

        #expect(model.value == "12.34 USDC")
    }
}
