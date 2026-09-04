// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
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
    func insufficientBalanceSheet() {
        let asset = Asset.mock()
        let requirement = BalanceRequirement(required: 2, available: 1, shortfall: 1)

        guard case let .balanceRequired(sheetAsset, _, sheetRequirement, _) = build(for: GemConfirmError.InsufficientBalance(asset: asset.map(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1))) else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        #expect(sheetAsset == asset)
        #expect(sheetRequirement == requirement)
    }

    @Test
    func minimumAccountBalanceSheet() {
        guard case let .accountMinimalBalance(_, required) = build(for: GemConfirmError.MinimumAccountBalanceTooLow(asset: Asset.mock().map(), requirement: GemBalanceRequirement(required: 10, available: 0, shortfall: 10))) else {
            Issue.record("Expected accountMinimalBalance sheet")
            return
        }
        #expect(required == BigInt(10))
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
    func unmappedErrorsBuildNoSheet() {
        let unknown = NSError(domain: "chain", code: 1, userInfo: [NSLocalizedDescriptionKey: "connection lost"])
        let chainErrorWithoutSheet = NSError(domain: "chain", code: 1, userInfo: [NSLocalizedDescriptionKey: "insufficient balance for transfer"])

        #expect(build(for: unknown) == nil)
        #expect(build(for: chainErrorWithoutSheet) == nil)
    }

    private func build(for error: Error) -> InfoSheetType? {
        ConfirmInfoSheetBuilder.build(
            for: ConfirmTransferError(error: error),
            feePrice: nil,
            currency: Currency.usd.rawValue,
            acquireFlow: { _ in .fiat },
            onGetAsset: { _, _ in },
        )
    }
}
