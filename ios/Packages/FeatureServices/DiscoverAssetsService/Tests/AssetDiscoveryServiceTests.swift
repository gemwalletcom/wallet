// Copyright (c). Gem Wallet. All rights reserved.

import AssetsServiceTestKit
import DiscoverAssetsService
import DiscoverAssetsServiceTestKit
import Foundation
import GemAPITestKit
import NFTServiceTestKit
import Preferences
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
import TransactionsServiceTestKit

struct AssetDiscoveryServiceTests {
    @Test
    func syncTransactionsOnceForImportedWallet() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let transactionStore = TransactionStore.mock(db: db)
        let nftStore = NFTStore.mock(db: db)
        let walletId = WalletId.mock()
        let wallet = Wallet.mock(
            id: walletId,
            accounts: [.mock(chain: .ethereum, address: walletId.address)],
            source: .import,
        )
        let preferences = WalletPreferences(walletId: wallet.id)
        defer { preferences.clear() }
        preferences.clear()

        try walletStore.addWallet(wallet)

        let assetId = AssetId.mock(.ethereum)
        let initialTransaction = Transaction(
            id: TransactionId(chain: .ethereum, hash: "transaction-1"),
            assetId: assetId,
            from: "0x0000000000000000000000000000000000000001",
            to: "0x0000000000000000000000000000000000000002",
            contract: nil,
            type: .transfer,
            state: .confirmed,
            blockNumber: "1",
            sequence: "1",
            fee: "1",
            feeAssetId: assetId,
            value: "1",
            memo: nil,
            direction: .incoming,
            utxoInputs: [],
            utxoOutputs: [],
            metadata: nil,
            createdAt: .now,
        )
        let collectionOneId = NFTCollectionId(chain: .ethereum, contractAddress: "collection-1")
        let assetOneId = NFTAssetId(chain: .ethereum, contractAddress: "collection-1", tokenId: "nft-1")
        let initialNFT = NFTData(
            collection: .mock(id: collectionOneId, chain: .ethereum),
            assets: [.mock(id: assetOneId, collectionId: collectionOneId, chain: .ethereum)],
        )
        let transactionProvider = GemAPITransactionServiceMock(
            walletTransactionsResponse: TransactionsResponse(transactions: [initialTransaction], addressNames: []),
        )
        let nftProvider = GemAPINFTServiceMock(nftAssets: [initialNFT])
        let service = AssetDiscoveryService.mock(
            assetsListService: GemAPIAssetsListServiceMock(assetsByDeviceIdResult: []),
            transactionsService: .mock(
                provider: transactionProvider,
                transactionStore: transactionStore,
                assetsService: .mock(
                    assetStore: .mock(db: db),
                    balanceStore: .mock(db: db),
                ),
                addressStore: .mock(db: db),
            ),
            nftService: .mock(apiService: nftProvider, nftStore: nftStore),
        )

        try await service.discoverAssets(wallet: wallet)

        let initialSavedTransactions = try transactionStore.getTransactions(states: [.confirmed])
        let initialSavedNFTs = try fetchNFTs(db: db, walletId: wallet.id)

        #expect(preferences.completeInitialLoadAssets)
        #expect(preferences.completeInitialLoadTransactions)
        #expect(preferences.completeInitialLoadNFTs)
        #expect(preferences.assetsTimestamp > 0)
        #expect(preferences.transactionsTimestamp > 0)
        #expect(initialSavedTransactions.map(\.id.hash) == [initialTransaction.id.hash])
        #expect(initialSavedNFTs.map(\.collection.id) == [initialNFT.collection.id])
        #expect(initialSavedNFTs.flatMap(\.assets).map(\.id) == initialNFT.assets.map(\.id))

        let nextTransaction = Transaction(
            id: TransactionId(chain: .ethereum, hash: "transaction-2"),
            assetId: assetId,
            from: "0x0000000000000000000000000000000000000003",
            to: "0x0000000000000000000000000000000000000004",
            contract: nil,
            type: .transfer,
            state: .confirmed,
            blockNumber: "2",
            sequence: "2",
            fee: "2",
            feeAssetId: assetId,
            value: "2",
            memo: nil,
            direction: .incoming,
            utxoInputs: [],
            utxoOutputs: [],
            metadata: nil,
            createdAt: .now,
        )
        let collectionTwoId = NFTCollectionId(chain: .ethereum, contractAddress: "collection-2")
        let assetTwoId = NFTAssetId(chain: .ethereum, contractAddress: "collection-2", tokenId: "nft-2")
        let nextNFT = NFTData(
            collection: .mock(id: collectionTwoId, chain: .ethereum),
            assets: [.mock(id: assetTwoId, collectionId: collectionTwoId, chain: .ethereum)],
        )

        transactionProvider.setWalletTransactionsResponse(
            TransactionsResponse(transactions: [nextTransaction], addressNames: []),
        )
        nftProvider.setNFTAssets([nextNFT])

        try await service.discoverAssets(wallet: wallet)
        let savedTransactions = try transactionStore.getTransactions(states: [.confirmed])
        let savedNFTs = try fetchNFTs(db: db, walletId: wallet.id)

        #expect(savedTransactions.map(\.id.hash) == [initialTransaction.id.hash])
        #expect(savedNFTs.map(\.collection.id) == [initialNFT.collection.id])
        #expect(savedNFTs.flatMap(\.assets).map(\.id) == initialNFT.assets.map(\.id))
    }

    private func fetchNFTs(db: DB, walletId: WalletId) throws -> [NFTData] {
        try db.dbQueue.read { database in
            try NFTRequest(walletId: walletId, filter: .all).fetch(database)
        }
    }
}
