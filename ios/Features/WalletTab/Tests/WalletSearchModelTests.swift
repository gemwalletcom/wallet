// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing
@testable import WalletTab

struct WalletSearchModelTests {
    @Test
    func searchMode() {
        var model = WalletSearchModel(selectType: .manage)
        #expect(model.searchMode(scope: .all) == .initial)
        #expect(model.searchMode(scope: .filter(.stablecoins)) == .tagBrowsing)

        model.searchableQuery = "bitcoin"
        #expect(model.searchMode(scope: .all) == .searching)
    }

    @Test
    func assetsLimit() {
        var model = WalletSearchModel(selectType: .manage)

        #expect(model.assetsLimit(scope: .all) == 12)
        #expect(model.assetsLimit(scope: .filter(.stablecoins)) == 18)

        model.searchableQuery = "bitcoin"
        #expect(model.assetsLimit(scope: .all) == 25)
    }

    @Test
    func fetchLimit() {
        var model = WalletSearchModel(selectType: .manage)

        #expect(model.fetchLimit(scope: .all) == 13)
        #expect(model.fetchLimit(scope: .filter(.stablecoins)) == 19)

        model.searchableQuery = "bitcoin"
        #expect(model.fetchLimit(scope: .all) == 100)
    }

    @Test
    func staticMembers() {
        #expect(WalletSearchModel.initialFetchLimit == 13)
        #expect(WalletSearchModel.searchItemTypes == [.asset, .perpetual, .list])
        #expect(WalletSearchModel.recentActivityTypes == RecentActivityType.allCases)
    }
}
