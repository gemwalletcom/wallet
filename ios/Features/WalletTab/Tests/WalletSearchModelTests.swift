// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing
@testable import WalletTab

struct WalletSearchModelTests {
    @Test
    func staticMembers() {
        #expect(WalletSearchModel.searchItemTypes == [.asset, .perpetual, .list, .nft])
    }
}
