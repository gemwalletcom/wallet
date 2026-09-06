// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmMemoViewModelTests {
    @Test
    func cosmos() {
        let asset = Asset.mock(id: AssetId(chain: .cosmos, tokenId: nil))
        let memo = "test memo"
        let model = ConfirmMemoViewModel(transfer: .mock(type: .transfer(asset), recipient: .mock(memo: memo)))

        guard case let .memo(item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.memo)
        #expect(item.subtitle == memo)
    }

    @Test
    func stellar() {
        let asset = Asset.mock(id: AssetId(chain: .stellar, tokenId: nil))
        let memo = "stellar memo"
        let model = ConfirmMemoViewModel(transfer: .mock(type: .deposit(asset), recipient: .mock(memo: memo)))

        guard case let .memo(item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.memo)
        #expect(item.subtitle == memo)
    }

    @Test
    func ton() {
        let asset = Asset.mock(id: AssetId(chain: .ton, tokenId: nil))
        let memo = "ton comment"
        let model = ConfirmMemoViewModel(transfer: .mock(type: .withdrawal(asset), recipient: .mock(memo: memo)))

        guard case let .memo(item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.memo)
        #expect(item.subtitle == memo)
    }

    @Test
    func emptyMemo() {
        let asset = Asset.mock(id: AssetId(chain: .solana, tokenId: nil))
        let model = ConfirmMemoViewModel(transfer: .mock(type: .transfer(asset), recipient: .mock()))

        if case let .memo(item) = model.itemModel {
            #expect(item.subtitle == "-")
        } else {
            Issue.record("Expected memo item model for empty memo")
        }
    }
}
