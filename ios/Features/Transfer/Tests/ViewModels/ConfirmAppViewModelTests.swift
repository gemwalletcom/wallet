// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmAppViewModelTests {
    @Test
    func generic() {
        let appMetadata = TransactionAppMetadata.mock(
            name: "PancakeSwap - Trade",
            url: "https://pancakeswap.finance/swap",
        )
        let model = ConfirmAppViewModel(type: .generic(asset: .mock(), appMetadata: appMetadata, extra: .mock()))

        guard case let .app(item) = model.itemModel else { return }
        #expect(item.title == Localized.WalletConnect.app)
        #expect(item.subtitle == "PancakeSwap")
    }
}
