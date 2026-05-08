// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing
@testable import WalletTab

struct WalletSearchModelTests {
    @Test
    func searchMode() {
        var model = WalletSearchModel(selectType: .manage)
        #expect(model.searchMode(tag: nil) == .initial)
        #expect(model.searchMode(tag: "stablecoins") == .tagBrowsing)

        model.searchableQuery = "bitcoin"
        #expect(model.searchMode(tag: nil) == .searching)
    }

    @Test
    func assetsLimit() {
        var model = WalletSearchModel(selectType: .manage)

        #expect(model.assetsLimit(tag: nil) == 12)
        #expect(model.assetsLimit(tag: "stablecoins") == 18)

        model.searchableQuery = "bitcoin"
        #expect(model.assetsLimit(tag: nil) == 25)
    }

    @Test
    func fetchLimit() {
        var model = WalletSearchModel(selectType: .manage)

        #expect(model.fetchLimit(tag: nil) == 13)
        #expect(model.fetchLimit(tag: "stablecoins") == 19)

        model.searchableQuery = "bitcoin"
        #expect(model.fetchLimit(tag: nil) == 100)
    }

    @Test
    func staticMembers() {
        #expect(WalletSearchModel.initialFetchLimit == 13)
        #expect(WalletSearchModel.searchItemTypes == [.asset, .perpetual])
        #expect(WalletSearchModel.recentActivityTypes == RecentActivityType.allCases)
    }
}
