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
    @Test
    func updateBalanceSkipsPrefixCollidingChainToken() async throws {
        let bitcoin = Chain.bitcoin.assetId
        let bitcoinCashToken = AssetId(chain: .bitcoinCash, tokenId: "token")

        let db = DB.mockWithChains([.bitcoin, .bitcoinCash])
        try AssetStore(db: db).add(assets: [.mock(asset: .mock(id: bitcoinCashToken))])
        let wallet = Wallet.mock(accounts: [.mock(chain: .bitcoin, address: "bitcoin-address")])
        try WalletStore(db: db).addWallet(wallet)
        let balanceStore = BalanceStore.mock(db: db)
        let chainService = ChainServiceMock()
        chainService.coinBalances["bitcoin-address"] = AssetBalance(assetId: bitcoin, balance: .mock(available: 100))
        chainService.tokenBalances["bitcoin-address"] = [AssetBalance(assetId: bitcoinCashToken, balance: .mock(available: 50))]
        let service = BalanceService.mock(
            balanceStore: balanceStore,
            assetsService: .mock(assetStore: .mock(db: db), balanceStore: balanceStore),
            chainServiceFactory: ChainServiceFactoryMock(chainService: chainService),
        )

        await service.updateBalance(for: wallet, assetIds: [bitcoin, bitcoinCashToken])

        #expect(try balanceStore.getBalance(walletId: wallet.id, assetId: bitcoin)?.available == 100)
        #expect(try balanceStore.getBalance(walletId: wallet.id, assetId: bitcoinCashToken) == nil)
    }
}
