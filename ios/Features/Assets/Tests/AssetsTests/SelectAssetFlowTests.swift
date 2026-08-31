// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import class Gemstone.GemAssetConfigService
import Primitives
import Store
import Testing

struct SelectAssetFlowTests {
    let assetConfig = GemAssetConfigService()

    @Test
    func rowSelection() {
        #expect(SelectAssetType.send(.none).flow(assetConfig: assetConfig).rowSelection == .navigate)
        #expect(SelectAssetType.receive(.asset).flow(assetConfig: assetConfig).rowSelection == .navigate)
        #expect(SelectAssetType.receive(.collection).flow(assetConfig: assetConfig).rowSelection == .navigate)
        #expect(SelectAssetType.buy.flow(assetConfig: assetConfig).rowSelection == .navigate)
        #expect(SelectAssetType.deposit.flow(assetConfig: assetConfig).rowSelection == .navigate)
        #expect(SelectAssetType.withdraw.flow(assetConfig: assetConfig).rowSelection == .navigate)
        #expect(SelectAssetType.manage.flow(assetConfig: assetConfig).rowSelection == .toggle)
        #expect(SelectAssetType.swap(.pay).flow(assetConfig: assetConfig).rowSelection == .select)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flow(assetConfig: assetConfig).rowSelection == .select)
        #expect(SelectAssetType.priceAlert.flow(assetConfig: assetConfig).rowSelection == .select)
    }

    @Test
    func selectionEffect() {
        #expect(SelectAssetType.priceAlert.flow(assetConfig: assetConfig).selectionEffect == .enablePriceAlert)
        #expect(SelectAssetType.swap(.pay).flow(assetConfig: assetConfig).selectionEffect == .recordRecent)
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flow(assetConfig: assetConfig).selectionEffect == .recordRecent)
        #expect(SelectAssetType.receive(.asset).flow(assetConfig: assetConfig).selectionEffect == .recordRecent)
        #expect(SelectAssetType.receive(.collection).flow(assetConfig: assetConfig).selectionEffect == .recordRecent)
        #expect(SelectAssetType.buy.flow(assetConfig: assetConfig).selectionEffect == .recordRecent)
    }

    @Test
    func capabilities() {
        #expect(SelectAssetType.send(.none).flow(assetConfig: assetConfig).capabilities == [.chainFilter, .recents])
        #expect(SelectAssetType.receive(.asset).flow(assetConfig: assetConfig).capabilities == [.networkSearch, .chainFilter, .recents])
        #expect(SelectAssetType.receive(.collection).flow(assetConfig: assetConfig).capabilities == [.networkSearch, .recents])
        #expect(SelectAssetType.buy.flow(assetConfig: assetConfig).capabilities == [.networkSearch, .chainFilter, .recents, .popularSection])
        #expect(SelectAssetType.swap(.pay).flow(assetConfig: assetConfig).capabilities == [.chainFilter, .recents])
        #expect(SelectAssetType.swap(.receive(chains: [], assetIds: [])).flow(assetConfig: assetConfig).capabilities == [.networkSearch, .chainFilter, .recents])
        #expect(SelectAssetType.manage.flow(assetConfig: assetConfig).capabilities == [.networkSearch, .chainFilter, .balanceFilter, .addCustomToken])
        #expect(SelectAssetType.priceAlert.flow(assetConfig: assetConfig).capabilities == [.networkSearch, .chainFilter, .popularSection])
        #expect(SelectAssetType.deposit.flow(assetConfig: assetConfig).capabilities.isEmpty == true)
        #expect(SelectAssetType.withdraw.flow(assetConfig: assetConfig).capabilities == [.depositAssetDisplay])
    }

    @Test
    func defaultFilters() {
        #expect(SelectAssetType.send(.none).flow(assetConfig: assetConfig).defaultFilters == [.enabled, .hasBalance])
        #expect(SelectAssetType.receive(.asset).flow(assetConfig: assetConfig).defaultFilters == [.enabled])
        #expect(SelectAssetType.buy.flow(assetConfig: assetConfig).defaultFilters == [.enabled, .buyable])
        #expect(SelectAssetType.swap(.pay).flow(assetConfig: assetConfig).defaultFilters == [.enabled, .swappable, .hasAvailableBalance])
        #expect(SelectAssetType.manage.flow(assetConfig: assetConfig).defaultFilters == [.enabled])
        #expect(SelectAssetType.priceAlert.flow(assetConfig: assetConfig).defaultFilters == [.enabled, .priceAlerts])
    }

    @Test
    func collectionFiltersRestrictToNftChains() {
        let filters = SelectAssetType.receive(.collection).flow(assetConfig: assetConfig).defaultFilters
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
        )).flow(assetConfig: assetConfig).defaultFilters

        #expect(filters == [
            .enabled,
            .swappable,
            .chainsOrAssets(["ethereum"], ["smartchain_0x123"]),
        ])
    }
}
