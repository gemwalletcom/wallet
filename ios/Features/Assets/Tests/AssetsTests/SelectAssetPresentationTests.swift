// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import GemstonePrimitives
import Primitives
import Store
import Testing

struct SelectAssetPresentationTests {
    @Test
    func flowTypeMapsEverySelectType() {
        #expect(SelectAssetType.send(.none).flowType == .send)
        #expect(SelectAssetType.receive(.asset).flowType == .receive)
        #expect(SelectAssetType.receive(.collection).flowType == .receiveCollection)
        #expect(SelectAssetType.buy.flowType == .buy)
        #expect(SelectAssetType.swap(.pay).flowType == .swapPay)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flowType == .swapReceive)
        #expect(SelectAssetType.manage.flowType == .manage)
        #expect(SelectAssetType.priceAlert.flowType == .priceAlert)
        #expect(SelectAssetType.deposit.flowType == .deposit)
        #expect(SelectAssetType.withdraw.flowType == .withdraw)
    }

    @Test
    func actionComesFromTheFlow() {
        #expect(SelectAssetType.send(.none).action == .send)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).action == .swapReceive)
        #expect(SelectAssetType.manage.action == nil)
        #expect(SelectAssetType.priceAlert.flow.enablesPriceAlert)
        #expect(SelectAssetType.manage.flow.rowAction == .toggle)
    }

    @Test
    func defaultFilters() {
        #expect(SelectAssetType.send(.none).presentation().defaultFilters == [.enabled, .hasBalance])
        #expect(SelectAssetType.receive(.asset).presentation().defaultFilters == [.enabled])
        #expect(SelectAssetType.buy.presentation().defaultFilters == [.enabled, .buyable])
        #expect(SelectAssetType.swap(.pay).presentation().defaultFilters == [.enabled, .swappable, .hasAvailableBalance])
        #expect(SelectAssetType.manage.presentation().defaultFilters == [.enabled])
        #expect(SelectAssetType.priceAlert.presentation().defaultFilters == [.enabled, .priceAlerts])
    }

    @Test
    func collectionFiltersRestrictToNftChains() {
        let filters = SelectAssetType.receive(.collection).presentation().defaultFilters
        let chains = filters.flatMap { filter -> [String] in
            guard case let .chainsOrAssets(_, assets) = filter else { return [] }
            return assets
        }

        #expect(filters.contains(.enabled) == true)
        #expect(chains.contains(Chain.ethereum.rawValue) == true)
        #expect(chains.contains(Chain.bitcoin.rawValue) == false)
    }

    @Test
    func swapReceiveFiltersCarryPairConstraints() {
        let filters = SelectAssetType.swap(.receive(
            chains: [.ethereum],
            assetIds: [AssetId(chain: .smartChain, tokenId: "0x123")],
        )).presentation().defaultFilters

        #expect(filters == [
            .enabled,
            .swappable,
            .chainsOrAssets(["ethereum"], ["smartchain_0x123"]),
        ])
    }
}
