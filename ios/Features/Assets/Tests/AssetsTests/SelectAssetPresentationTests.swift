// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import enum Gemstone.GemAssetFilter
import enum Gemstone.GemSelectAssetType
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
        #expect(SelectAssetType.swap(.receive(payAssetId: AssetId(chain: .ethereum))).flowType == .swapReceive(payAssetId: "ethereum"))
        #expect(SelectAssetType.payment([AssetId(chain: .ethereum)]).flowType == .payment(assetIds: ["ethereum"]))
        #expect(SelectAssetType.manage.flowType == .manage)
        #expect(SelectAssetType.priceAlert.flowType == .priceAlert)
        #expect(SelectAssetType.deposit.flowType == .deposit)
        #expect(SelectAssetType.withdraw.flowType == .withdraw)
    }

    @Test
    func filtersAndScopeMapFromTheFlow() {
        #expect(GemSelectAssetType.send.flow().requestFilters == [.enabled, .hasBalance])
        #expect(GemSelectAssetType.send.flow().requestScope == .wallet)
        #expect(GemSelectAssetType.priceAlert.flow().requestScope == .allAssets)
    }

    @Test
    func chainsOrAssetIdsFilterMapsBothSlots() {
        let filter = AssetsRequestFilter(core: .chainsOrAssetIds(chains: ["ethereum"], assetIds: ["smartchain_0x123"]))

        #expect(filter == .chainsOrAssets(["ethereum"], ["smartchain_0x123"]))
    }
}
