// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing
@testable import WalletTab

struct WalletSearchModelTests {
    @Test
    func searchMode() {
        var model = WalletSearchModel()
        #expect(model.searchMode == .initial)

        model.searchableQuery = "bitcoin"
        #expect(model.searchMode == .searching)
    }

    @Test
    func assetsLimit() {
        var model = WalletSearchModel()

        #expect(model.assetsLimit == 12)

        model.searchableQuery = "bitcoin"
        #expect(model.assetsLimit == 25)
    }

    @Test
    func fetchLimit() {
        var model = WalletSearchModel()

        #expect(model.fetchLimit == 13)

        model.searchableQuery = "bitcoin"
        #expect(model.fetchLimit == 100)
    }

    @Test
    func staticMembers() {
        #expect(WalletSearchModel.initialFetchLimit == 13)
        #expect(WalletSearchModel.searchItemTypes == [.asset, .perpetual, .list, .nft])
    }
}
