// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.confirmErrorInfo
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

@MainActor
struct ConfirmInfoSheetBuilderTests {
    @Test
    func insufficientBalanceSheetCarriesTheAmountsCoreFormatted() {
        let asset = Asset.mockEthereum()
        let error = GemConfirmError.InsufficientBalance(
            asset: asset.toGem(),
            requirement: GemBalanceRequirement(required: 2_000_000_000_000_000_000, available: 1_000_000_000_000_000_000, shortfall: 1_000_000_000_000_000_000),
        )

        guard case let .balanceRequired(info, _, _) = build(for: error) else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        #expect(info.asset?.id == asset.id.identifier)
        #expect(info.required?.value == 2)
        #expect(info.available?.value == 1)
        #expect(info.shortfall?.value == 1)
    }

    @Test
    func minimumAccountBalanceSheet() {
        let error = GemConfirmError.MinimumAccountBalanceTooLow(
            asset: Asset.mockEthereum().toGem(),
            requirement: GemBalanceRequirement(required: 100_000_000_000_000_000, available: 0, shortfall: 100_000_000_000_000_000),
        )

        guard case let .accountMinimalBalance(info) = build(for: error) else {
            Issue.record("Expected accountMinimalBalance sheet")
            return
        }
        #expect(info.required?.value == 0.1)
    }

    @Test
    func swapBelowMinimumSheetNamesTheProvider() {
        let asset = Asset.mockEthereum()
        let error = GemConfirmError.BelowSwapMinimum(
            asset: asset.toGem(),
            provider: .nearIntents,
            providerName: "NEAR Intents",
            requirement: GemBalanceRequirement(required: 2_000_000_000_000_000_000, available: 1_500_000_000_000_000_000, shortfall: 500_000_000_000_000_000),
        )

        guard case let .swapMinimumAmount(info, providerName, _, _) = build(for: error) else {
            Issue.record("Expected swapMinimumAmount sheet")
            return
        }
        #expect(providerName == "NEAR Intents")
        #expect(info.required?.value == 2)
        #expect(info.shortfall?.value == 0.5)
    }

    @Test
    func dustThresholdSheet() {
        let error = GemConfirmError.Sign(error: .dustThreshold, chain: Chain.bitcoin.rawValue, msg: "message can change")

        guard case let .dustThreshold(chain, _) = build(for: error) else {
            Issue.record("Expected dustThreshold sheet")
            return
        }
        #expect(chain == .bitcoin)
    }

    @Test
    func anErrorTheUserCannotActOnBuildsNoSheet() {
        #expect(info(for: .Offline) == nil)
        #expect(info(for: .Cancelled) == nil)
    }

    private func info(for error: GemConfirmError) -> InfoSheetType? {
        confirmErrorInfo(error: error, prices: [], currency: Currency.usd.toGem()).map {
            ConfirmInfoSheetBuilder.build(for: $0, networkFeeBuyAmount: 10, onGetAsset: { _, _ in })
        }
    }

    private func build(for error: GemConfirmError) -> InfoSheetType? {
        info(for: error)
    }
}
