// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import InfoSheet
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import Validators

@MainActor
struct ConfirmInfoSheetBuilderTests {
    @Test
    func insufficientBalanceSheet() {
        let asset = Asset.mock()

        guard case let .insufficientBalance(sheetAsset, _) = build(for: TransferAmountCalculatorError.insufficientBalance(asset)) else {
            Issue.record("Expected insufficientBalance sheet")
            return
        }
        #expect(sheetAsset == asset)
    }

    @Test
    func minimumAccountBalanceSheet() {
        guard case let .accountMinimalBalance(_, required) = build(for: TransferAmountCalculatorError.minimumAccountBalanceTooLow(.mock(), required: BigInt(10))) else {
            Issue.record("Expected accountMinimalBalance sheet")
            return
        }
        #expect(required == BigInt(10))
    }

    @Test
    func dustThresholdSheet() {
        let asset = Asset.mock()
        let error = NSError(domain: "chain", code: 1, userInfo: [NSLocalizedDescriptionKey: "dust threshold not met"])

        guard case let .dustThreshold(chain, _) = build(for: error, asset: asset) else {
            Issue.record("Expected dustThreshold sheet")
            return
        }
        #expect(chain == asset.chain)
    }

    @Test
    func unmappedErrorsBuildNoSheet() {
        let unknown = NSError(domain: "chain", code: 1, userInfo: [NSLocalizedDescriptionKey: "connection lost"])
        let chainErrorWithoutSheet = NSError(domain: "chain", code: 1, userInfo: [NSLocalizedDescriptionKey: "insufficient balance for transfer"])

        #expect(build(for: unknown) == nil)
        #expect(build(for: chainErrorWithoutSheet) == nil)
    }

    private func build(for error: Error, asset: Asset = .mock()) -> InfoSheetType? {
        ConfirmInfoSheetBuilder.build(for: error, asset: asset, feePrice: nil, currency: Currency.usd.rawValue, onGetNetworkFeeAsset: {})
    }
}
