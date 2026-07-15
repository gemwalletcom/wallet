// Copyright (c). Gem Wallet. All rights reserved.

import AssetsServiceTestKit
import BalanceService
import BalanceServiceTestKit
import BlockchainTestKit
import ChainServiceTestKit
import Foundation
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct BalanceServiceTests {
    // `seievm` (Chain.seiEvm) has `sei` (Chain.sei) as a rawValue prefix, so a
    // string-prefix asset match pulls a Sei-EVM token into the Sei account's
    // token fetch. Matching by typed chain must skip it. Real colliding pairs in
    // the current chain set: bitcoin/bitcoincash and sei/seievm.
    @Test
    func updateBalanceSkipsPrefixCollidingChainToken() async throws {
        let sei = Chain.sei.assetId
        let seiEvmToken = AssetId(chain: .seiEvm, tokenId: "0xtoken")

        let db = DB.mockWithChains([.sei, .seiEvm])
        try AssetStore(db: db).add(assets: [.mock(asset: .mock(id: seiEvmToken))])
        let wallet = Wallet.mock(accounts: [.mock(chain: .sei, address: "sei-address")])
        try WalletStore(db: db).addWallet(wallet)
        let balanceStore = BalanceStore.mock(db: db)
        let chainService = ChainServiceMock()
        chainService.coinBalances["sei-address"] = AssetBalance(assetId: sei, balance: .mock(available: 100))
        chainService.tokenBalances["sei-address"] = [AssetBalance(assetId: seiEvmToken, balance: .mock(available: 50))]
        let service = BalanceService.mock(
            balanceStore: balanceStore,
            assetsService: .mock(assetStore: .mock(db: db), balanceStore: balanceStore),
            chainServiceFactory: ChainServiceFactoryMock(chainService: chainService),
        )

        await service.updateBalance(for: wallet, assetIds: [sei, seiEvmToken])

        #expect(try balanceStore.getBalance(walletId: wallet.id, assetId: sei)?.available == 100)
        #expect(try balanceStore.getBalance(walletId: wallet.id, assetId: seiEvmToken) == nil)
    }
}
