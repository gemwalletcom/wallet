// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import func Gemstone.walletRow
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmSenderViewModelTests {
    @Test
    func wallet() {
        let wallet = Wallet.mock()
        let model = ConfirmSenderViewModel(row: walletRow(wallet: wallet.toGem()))

        guard case let .sender(item) = model.itemModel else { return }
        #expect(item.title == Localized.Common.wallet)
        #expect(item.subtitle == wallet.name)
        #expect(item.imageStyle != nil)
    }
}
