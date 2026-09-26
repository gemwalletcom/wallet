// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import enum Gemstone.FeePriority
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

extension NetworkFeeSceneViewModel {
    var selectedRowItem: ListItemModel? {
        feeRateRows.first(where: \.isSelected).map(rowItem(for:))
    }
}

private extension GemFeeRateRow {
    static func rate(_ priority: Gemstone.FeePriority = .normal, fee: BigInt? = nil, isSelected: Bool = true) -> GemFeeRateRow {
        .mock(
            kind: .priority(priority: priority),
            fee: fee,
            value: .feeRate(rate: .mock(value: 1, unit: .plain, display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest), unit: .gwei),
            isSelected: isSelected,
        )
    }
}

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
            rows: [.rate()],
            showsOptions: false,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 1,
            normalTotal: 1,
        )).showFeeRates == false)
        #expect(NetworkFeeSceneViewModel.mock(feeRates: .mock(
            rows: [.rate(), .rate(.fast, isSelected: false)],
            showsOptions: true,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 1,
            normalTotal: 1,
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
        #expect(selectable.showFeeDetails)
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
    func showFeeDetailsForLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(
            feeRates: .mock(
                rows: [.rate()],
                showsOptions: false,
                unitType: .gwei,
                unitDecimals: 9,
                selectedTotal: 1,
                normalTotal: 1,
            ),
            feeAmount: BigInt(1_000_000_000_000_000),
        )

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForMultipleRates() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: .mock(
            rows: [.rate(), .rate(.fast, isSelected: false)],
            showsOptions: true,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 1,
            normalTotal: 1,
        ))

        #expect(model.showFeeRates)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForSingleRateWhileReloading() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: .mock(
            rows: [.rate()],
            showsOptions: false,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 1,
            normalTotal: 1,
        ))

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func hideFeeDetailsBeforePreload() {
        #expect(NetworkFeeSceneViewModel.mock().showFeeDetails == false)
    }

    @Test
    func aRateRowShowsTheCoreRateWithItsLocalizedUnit() {
        let rate = GemFormattedNumber.mock(value: 2.5, unit: .plain, display: .number(precision: .fraction(min: 0, max: 1)), notation: .plain, tone: .plain, rounding: .toNearest)
        let solRate = GemFormattedNumber.mock(value: 2.5, unit: .symbol(symbol: "SOL"), display: .number(precision: .fraction(min: 0, max: 1)), notation: .plain, tone: .plain, rounding: .toNearest)
        let valueText = { (value: GemLocalizedText) in
            NetworkFeeSceneViewModel.mock(feeRates: .mock(
                rows: [.mock(value: value, isSelected: true)],
                showsOptions: false,
                unitType: .gwei,
                unitDecimals: 9,
                selectedTotal: 1,
                normalTotal: 1,
            )).selectedRowItem?.subtitle
        }

        #expect(valueText(.feeRate(rate: rate, unit: .gwei)) == "2.5 gwei")
        #expect(valueText(.feeRate(rate: rate, unit: .satVb)) == "2.5 sat/vB")
        #expect(valueText(.feeRate(rate: solRate, unit: .native)) == "2.5 SOL")
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9),
            feeRates: .mock(
                rows: [.rate(fee: 5000)],
                showsOptions: false,
                unitType: .native,
                unitDecimals: 9,
                selectedTotal: 5000,
                normalTotal: 5000,
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
                rows: [.rate(fee: 21_000_000_000_000)],
                showsOptions: false,
                unitType: .gwei,
                unitDecimals: 9,
                selectedTotal: 1,
                normalTotal: 1,
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
                rows: [.rate(fee: 5000)],
                showsOptions: false,
                unitType: .native,
                unitDecimals: 9,
                selectedTotal: 5000,
                normalTotal: 5000,
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
    func customFeeInputConfirmsEnteredRate() throws {
        let custom = try #require(NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(),
            feeRates: .mock(
                rows: [.rate(fee: 1000)],
                showsOptions: false,
                unitType: .satVb,
                unitDecimals: 1,
                selectedTotal: 20,
                normalTotal: 20,
            ),
            feeAmount: 1000,
        ).customFeeModel())

        #expect(custom.isConfirmEnabled == false)
        custom.input = "4"
        #expect(custom.isConfirmEnabled)
    }

    @Test
    func customFeeInputRejectsRateAboveMax() throws {
        let custom = try #require(NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(),
            feeRates: .mock(
                rows: [.rate(fee: 1000)],
                showsOptions: false,
                unitType: .satVb,
                unitDecimals: 1,
                selectedTotal: 20,
                normalTotal: 20,
            ),
            feeAmount: 1000,
        ).customFeeModel())
        custom.input = "999"

        #expect(custom.isConfirmEnabled == false)
        #expect(custom.errorText != nil)
    }

    @Test
    func customFeeConfirmForwardsSelectionToOwner() async {
        await confirmation { selected in
            let custom = NetworkFeeSceneViewModel.mock(
                feeAsset: .mock(),
                feeRates: .mock(
                    rows: [.rate(fee: 1000)],
                    showsOptions: false,
                    unitType: .satVb,
                    unitDecimals: 1,
                    selectedTotal: 20,
                    normalTotal: 20,
                ),
                feeAmount: 1000,
                onSelect: {
                    #expect($0 == .custom(gasPrice: 40))
                    selected()
                },
            ).customFeeModel()!
            custom.input = "4"
            custom.confirm()
        }
    }

    @Test
    func customFeeRejectedRateDoesNotConfirm() async {
        await confirmation(expectedCount: 0) { selected in
            let custom = NetworkFeeSceneViewModel.mock(
                feeAsset: .mock(),
                feeRates: .mock(
                    rows: [.rate(fee: 1000)],
                    showsOptions: false,
                    unitType: .satVb,
                    unitDecimals: 1,
                    selectedTotal: 20,
                    normalTotal: 20,
                ),
                feeAmount: 1000,
                onSelect: { _ in selected() },
            ).customFeeModel()!
            custom.input = "999"
            custom.confirm()
        }
    }

    @Test
    func customFeeMaxAnchoredToNormalRate() async throws {
        await confirmation { selected in
            let custom = NetworkFeeSceneViewModel.mock(
                feeAsset: .mock(),
                feeRates: .mock(
                    rows: [.rate(fee: 1000)],
                    showsOptions: false,
                    unitType: .satVb,
                    unitDecimals: 1,
                    selectedTotal: 20,
                    normalTotal: 20,
                ),
                feeAmount: 1000,
                onSelect: {
                    #expect($0 == .custom(gasPrice: 200))
                    selected()
                },
            ).customFeeModel()!
            custom.input = "20"
            custom.confirm()
        }

        let reopened = try #require(NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(),
            feeRates: .mock(
                rows: [.rate(fee: 1000)],
                showsOptions: false,
                unitType: .satVb,
                unitDecimals: 1,
                selectedTotal: 200,
                normalTotal: 20,
            ),
            feeAmount: 1000,
        ).customFeeModel())
        reopened.input = "21"
        #expect(reopened.isConfirmEnabled == false)
        #expect(reopened.errorText != nil)
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

        #expect(model.value == feeAsset.feeText(feeAmount))
        #expect(model.value != Asset.mock(id: .mock(chain: .hyperCore), name: "Hyperliquid", symbol: "HYPE", decimals: 8).feeText(feeAmount))
    }
}

extension Asset {
    func feeText(_ value: BigInt) -> String {
        ValueFormatter.auto.string(value, asset: self)
    }
}
