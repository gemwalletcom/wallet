// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct StakeStoreTests {
    @Test
    func deactivateValidatorsKeepsRow() throws {
        let store = StakeStore(db: .mockWithChains([.cosmos]))
        let elected = DelegationValidator.mock(.cosmos, id: "elected")
        let dropped = DelegationValidator.mock(.cosmos, id: "dropped")
        try store.updateValidators([elected, dropped])

        try store.deactivateValidators(assetId: Chain.cosmos.assetId, validatorIds: [dropped.id])

        let active = try store.getValidatorsActive(assetId: Chain.cosmos.assetId, providerType: .stake)
        #expect(active.map(\.id) == [elected.id])

        let stored = try store.getValidators(assetId: Chain.cosmos.assetId, providerType: .stake)
        #expect(stored.count == 2)
        #expect(stored.first { $0.id == dropped.id }?.apr == 0)
        #expect(stored.first { $0.id == elected.id }?.apr == elected.apr)
    }

    @Test
    func deactivateValidatorsKeepsOtherChain() throws {
        let store = StakeStore(db: .mockWithChains([.cosmos, .celestia]))
        let sharedId = "valoper1shared"
        try store.updateValidators([
            DelegationValidator.mock(.cosmos, id: sharedId),
            DelegationValidator.mock(.celestia, id: sharedId),
        ])

        try store.deactivateValidators(assetId: Chain.cosmos.assetId, validatorIds: [sharedId])

        let cosmos = try store.getValidatorsActive(assetId: Chain.cosmos.assetId, providerType: .stake)
        #expect(cosmos.isEmpty)

        let celestia = try store.getValidatorsActive(assetId: Chain.celestia.assetId, providerType: .stake)
        #expect(celestia.map(\.id) == [sharedId])
    }
}
