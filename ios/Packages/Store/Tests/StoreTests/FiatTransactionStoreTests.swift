// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct FiatTransactionStoreTests {
    private let wallet = Wallet.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .smartChain), .mock(chain: .tron), .mock(chain: .ethereum)])
    private let walletAssets: [AssetBasic] = [
        .mock(asset: .mock(name: "Bitcoin", symbol: "BTC", decimals: 8), properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true)),
        .mock(
            asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
        .mock(asset: .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6), properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true)),
        .mock(
            asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
        .mock(
            asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
    ]
    private let walletBalances: [UpdateBalance] = [
        .mock(assetId: .mock(chain: .smartChain), available: 1),
        .mock(assetId: .mock(chain: .tron), available: 2),
        .mock(assetId: .mock(chain: .ethereum), available: 3),
        .mock(assetId: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), available: 4),
    ]

    @Test
    func setTransactionsReplacesWalletSnapshot() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let store = FiatTransactionStore(db: db)

        try store.setTransactions(walletId: .mock(), transactions: [
            .mock(transaction: .mock(id: "2", status: .complete)),
            .mock(transaction: .mock(id: "1", status: .failed)),
        ])

        #expect(try storedIds(db) == ["1", "2"])

        try store.setTransactions(walletId: .mock(), transactions: [
            .mock(transaction: .mock(id: "2", status: .complete)),
        ])

        #expect(try storedIds(db) == ["2"])
    }

    private func storedIds(_ db: DB) throws -> [String] {
        try db.dbQueue
            .read { try FiatTransactionsQuery(walletId: .mock()).fetch($0) }
            .map(\.id)
            .sorted()
    }
}
