// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import GemstonePrimitives
import Primitives
import Store
import Testing

struct SelectAssetFlowTests {
    @Test
    func rowSelection() {
        #expect(SelectAssetType.send(.none).flow().rowSelection == .navigate)
        #expect(SelectAssetType.receive(.asset).flow().rowSelection == .navigate)
        #expect(SelectAssetType.receive(.collection).flow().rowSelection == .navigate)
        #expect(SelectAssetType.buy.flow().rowSelection == .navigate)
        #expect(SelectAssetType.deposit.flow().rowSelection == .navigate)
        #expect(SelectAssetType.withdraw.flow().rowSelection == .navigate)
        #expect(SelectAssetType.manage.flow().rowSelection == .toggle)
        #expect(SelectAssetType.swap(.pay).flow().rowSelection == .select)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flow().rowSelection == .select)
        #expect(SelectAssetType.priceAlert.flow().rowSelection == .select)
    }

    @Test
    func selectionEffect() {
        #expect(SelectAssetType.priceAlert.flow().selectionEffect == .enablePriceAlert)
        #expect(SelectAssetType.swap(.pay).flow().selectionEffect == .recordRecent)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flow().selectionEffect == .recordRecent)
        #expect(SelectAssetType.receive(.asset).flow().selectionEffect == .recordRecent)
        #expect(SelectAssetType.receive(.collection).flow().selectionEffect == .recordRecent)
        #expect(SelectAssetType.buy.flow().selectionEffect == .recordRecent)
    }

    @Test
    func capabilities() {
        #expect(SelectAssetType.send(.none).flow().capabilities == [.chainFilter, .recents])
        #expect(SelectAssetType.receive(.asset).flow().capabilities == [.networkSearch, .chainFilter, .recents])
        #expect(SelectAssetType.receive(.collection).flow().capabilities == [.networkSearch, .recents])
        #expect(SelectAssetType.buy.flow().capabilities == [.networkSearch, .chainFilter, .recents, .popularSection])
        #expect(SelectAssetType.swap(.pay).flow().capabilities == [.chainFilter, .recents])
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flow().capabilities == [.networkSearch, .chainFilter, .recents])
        #expect(SelectAssetType.manage.flow().capabilities == [.networkSearch, .chainFilter, .balanceFilter, .addCustomToken])
        #expect(SelectAssetType.priceAlert.flow().capabilities == [.networkSearch, .chainFilter, .popularSection])
        #expect(SelectAssetType.deposit.flow().capabilities.isEmpty == true)
        #expect(SelectAssetType.withdraw.flow().capabilities == [.depositAssetDisplay])
    }

    @Test
    func defaultFilters() {
        #expect(SelectAssetType.send(.none).flow().defaultFilters == [.enabled, .hasBalance])
        #expect(SelectAssetType.receive(.asset).flow().defaultFilters == [.enabled])
        #expect(SelectAssetType.buy.flow().defaultFilters == [.enabled, .buyable])
        #expect(SelectAssetType.swap(.pay).flow().defaultFilters == [.enabled, .swappable, .hasAvailableBalance])
        #expect(SelectAssetType.manage.flow().defaultFilters == [.enabled])
        #expect(SelectAssetType.priceAlert.flow().defaultFilters == [.enabled, .priceAlerts])
    }

    @Test
    func collectionFiltersRestrictToNftChains() {
        let filters = SelectAssetType.receive(.collection).flow().defaultFilters
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
        )).flow().defaultFilters

        #expect(filters == [
            .enabled,
            .swappable,
            .chainsOrAssets(["ethereum"], ["smartchain_0x123"]),
        ])
    }
}
