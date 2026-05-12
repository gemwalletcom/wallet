// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

private extension UserDefaults {
    static func mock() -> UserDefaults {
        let suiteName = UUID().uuidString
        let defaults = UserDefaults(suiteName: suiteName)!
        defaults.removePersistentDomain(forName: suiteName)
        return defaults
    }
}

private extension DB {
    func insertLegacyWallet(
        id: String,
        type: WalletType = .multicoin,
        accounts: [Account] = [],
        order: Int = 0,
    ) throws {
        try dbQueue.write { db in
            try db.execute(
                sql: """
                    INSERT INTO wallets (id, name, type, "index", "order", isPinned, source)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                arguments: [id, "", type.rawValue, 0, order, false, WalletSource.create.rawValue],
            )

            for account in accounts {
                try db.execute(
                    sql: """
                        INSERT INTO wallets_accounts (walletId, chain, address, extendedPublicKey, "index", derivationPath)
                        VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    arguments: [
                        id,
                        account.chain.rawValue,
                        account.address,
                        account.extendedPublicKey ?? "",
                        0,
                        account.derivationPath,
                    ],
                )
            }
        }
    }
}

@Suite(.serialized)
struct WalletIdMigrationTests {
    private let currentWalletKey = "currentWallet"

    @Test
    func migrateMulticoinWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-multicoin-1"
        let ethAddress = "0x1234567890abcdef"
        try db.insertLegacyWallet(
            id: oldId,
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress), .mock(chain: .bitcoin)],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .multicoin(address: ethAddress))
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func migrateViewWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-view-1"
        let address = "0xviewaddress"
        try db.insertLegacyWallet(
            id: oldId,
            type: .view,
            accounts: [.mock(chain: .ethereum, address: address)],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .view(chain: .ethereum, address: address))
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func migrateSingleWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-single-1"
        let address = "bc1qsingleaddress"
        try db.insertLegacyWallet(
            id: oldId,
            type: .single,
            accounts: [.mock(chain: .bitcoin, address: address)],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .single(chain: .bitcoin, address: address))
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func migratePrivateKeyWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-pk-1"
        let address = "0xprivatekey"
        try db.insertLegacyWallet(
            id: oldId,
            type: .privateKey,
            accounts: [.mock(chain: .ethereum, address: address)],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .privateKey(chain: .ethereum, address: address))
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func removeDuplicateMulticoinWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let ethAddress = "0xsameaddress"

        try db.insertLegacyWallet(
            id: "uuid-1",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
            order: 0,
        )
        try db.insertLegacyWallet(
            id: "uuid-2",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
            order: 1,
        )
        try db.insertLegacyWallet(
            id: "uuid-3",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
            order: 2,
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .multicoin(address: ethAddress))
        #expect(wallets.first?.externalId == "uuid-1")
    }

    @Test
    func mixedWalletTypes() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin, .solana])
        let walletStore = WalletStore(db: db)

        try db.insertLegacyWallet(
            id: "uuid-multicoin",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: "0xmulti")],
            order: 0,
        )
        try db.insertLegacyWallet(
            id: "uuid-single",
            type: .single,
            accounts: [.mock(chain: .bitcoin, address: "bc1single")],
            order: 1,
        )
        try db.insertLegacyWallet(
            id: "uuid-view",
            type: .view,
            accounts: [.mock(chain: .solana, address: "solview")],
            order: 2,
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 3)

        let ids = Set(wallets.map(\.id))
        #expect(ids.contains(.multicoin(address: "0xmulti")))
        #expect(ids.contains(.single(chain: .bitcoin, address: "bc1single")))
        #expect(ids.contains(.view(chain: .solana, address: "solview")))
    }

    @Test
    func updateChildTableReferences() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let assetStore = AssetStore(db: db)

        let oldId = "uuid-with-balances"
        let ethAddress = "0xwithbalances"
        try db.insertLegacyWallet(
            id: oldId,
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
        )

        let asset = AssetBasic.mock(asset: .mock(id: .mockEthereum()))
        try assetStore.add(assets: [asset])
        try db.dbQueue.write { db in
            try db.execute(
                sql: "INSERT INTO balances (assetId, walletId, isEnabled) VALUES (?, ?, ?)",
                arguments: [asset.asset.id.identifier, oldId, true],
            )
        }

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let newWalletId = try WalletId.from(id: "multicoin_\(ethAddress)")
        let balances = try balanceStore.getBalances(walletId: newWalletId, assetIds: [asset.asset.id])
        #expect(balances.count == 1)
    }

    @Test
    func removeUnmappedInvalidLegacyWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let validAddress = "0xvalid"
        try db.insertLegacyWallet(
            id: "uuid-valid",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: validAddress)],
        )
        try db.insertLegacyWallet(
            id: "uuid-unmapped",
            type: .multicoin,
            accounts: [.mock(chain: .bitcoin, address: "bc1unmapped")],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.map(\.id) == [.multicoin(address: validAddress)])

        let storedIds = try db.dbQueue.read { db in
            try String.fetchAll(db, sql: "SELECT id FROM \(WalletRecord.databaseTableName)")
        }
        #expect(!storedIds.contains("uuid-unmapped"))
    }

    @Test
    func multipleDuplicateGroups() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let address1 = "0xaddress1"
        try db.insertLegacyWallet(id: "uuid-1a", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address1)], order: 0)
        try db.insertLegacyWallet(id: "uuid-1b", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address1)], order: 1)
        try db.insertLegacyWallet(id: "uuid-1c", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address1)], order: 2)

        let address2 = "0xaddress2"
        try db.insertLegacyWallet(id: "uuid-2a", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address2)], order: 3)
        try db.insertLegacyWallet(id: "uuid-2b", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address2)], order: 4)

        let address3 = "0xaddress3"
        try db.insertLegacyWallet(id: "uuid-3", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address3)], order: 5)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 3)

        let ids = Set(wallets.map(\.id))
        #expect(ids.contains(.multicoin(address: address1)))
        #expect(ids.contains(.multicoin(address: address2)))
        #expect(ids.contains(.multicoin(address: address3)))

        let wallet1 = wallets.first { $0.id == .multicoin(address: address1) }
        #expect(wallet1?.externalId == "uuid-1a")

        let wallet2 = wallets.first { $0.id == .multicoin(address: address2) }
        #expect(wallet2?.externalId == "uuid-2a")
    }

    @Test
    func duplicateViewWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let address = "0xviewaddr"
        try db.insertLegacyWallet(id: "uuid-view-1", type: .view, accounts: [.mock(chain: .ethereum, address: address)], order: 1)
        try db.insertLegacyWallet(id: "uuid-view-2", type: .view, accounts: [.mock(chain: .ethereum, address: address)], order: 0)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .view(chain: .ethereum, address: address))
        #expect(wallets.first?.externalId == "uuid-view-2")
    }

    @Test
    func duplicateSingleWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.bitcoin])
        let walletStore = WalletStore(db: db)

        let address = "bc1qsingle"
        try db.insertLegacyWallet(id: "uuid-single-1", type: .single, accounts: [.mock(chain: .bitcoin, address: address)], order: 2)
        try db.insertLegacyWallet(id: "uuid-single-2", type: .single, accounts: [.mock(chain: .bitcoin, address: address)], order: 0)
        try db.insertLegacyWallet(id: "uuid-single-3", type: .single, accounts: [.mock(chain: .bitcoin, address: address)], order: 1)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .single(chain: .bitcoin, address: address))
        #expect(wallets.first?.externalId == "uuid-single-2")
    }

    @Test
    func keepWalletWithLowestOrder() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let address = "0xordertest"
        try db.insertLegacyWallet(id: "uuid-order-5", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)], order: 5)
        try db.insertLegacyWallet(id: "uuid-order-2", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)], order: 2)
        try db.insertLegacyWallet(id: "uuid-order-8", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)], order: 8)
        try db.insertLegacyWallet(id: "uuid-order-1", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)], order: 1)
        try db.insertLegacyWallet(id: "uuid-order-3", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)], order: 3)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.externalId == "uuid-order-1")
    }

    @Test
    func accountsUpdatedAfterMigration() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-accounts"
        let ethAddress = "0xaccounts"
        let btcAddress = "bc1accounts"
        try db.insertLegacyWallet(
            id: oldId,
            type: .multicoin,
            accounts: [
                .mock(chain: .ethereum, address: ethAddress),
                .mock(chain: .bitcoin, address: btcAddress),
            ],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.accounts.count == 2)

        let chains = Set(wallets.first?.accounts.map(\.chain) ?? [])
        #expect(chains.contains(.ethereum))
        #expect(chains.contains(.bitcoin))
    }

    @Test
    func walletAlreadyInNewFormat() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let ethAddress = "0xalreadymigrated"
        let newFormatId = WalletId.multicoin(address: ethAddress)
        let wallet = Wallet.mock(
            id: newFormatId,
            externalId: "old-uuid",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == newFormatId)
        #expect(wallets.first?.externalId == "old-uuid")
    }

    @Test
    func emptyDatabase() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mock()

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let walletStore = WalletStore(db: db)
        let wallets = try walletStore.getWallets()
        #expect(wallets.isEmpty)
    }

    @Test
    func multipleAccountsConsistentSelection() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-multi-accounts"
        let btcAddress = "bc1bitcoin"
        let ethAddress = "0xethereum"
        try db.insertLegacyWallet(
            id: oldId,
            type: .single,
            accounts: [
                .mock(chain: .bitcoin, address: btcAddress),
                .mock(chain: .ethereum, address: ethAddress),
            ],
        )

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == .single(chain: .ethereum, address: ethAddress))
    }
}

@Suite(.serialized)
struct WalletIdMigrationPreferenceTests {
    private let currentWalletKey = "currentWallet"

    @Test
    func migrateCurrentWalletPreference() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])

        let oldId = "uuid-current"
        let ethAddress = "0xcurrent"
        try db.insertLegacyWallet(
            id: oldId,
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
        )

        userDefaults.set(oldId, forKey: currentWalletKey)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let newCurrentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(newCurrentWalletId == "multicoin_\(ethAddress)")
    }

    @Test
    func setCurrentWalletWhenNoneSet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])

        try db.insertLegacyWallet(id: "uuid-first", type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0xfirst")], order: 0)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let currentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(currentWalletId == "multicoin_0xfirst")
    }

    @Test
    func fallbackCurrentWalletWhenInvalid() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])

        userDefaults.set("deleted-wallet-id", forKey: currentWalletKey)

        try db.insertLegacyWallet(id: "uuid-fallback", type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0xfallback")], order: 0)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let currentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(currentWalletId == "multicoin_0xfallback")
    }

    @Test
    func preserveCurrentWalletWhenAlreadyMigrated() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let ethAddress = "0xalready"
        let newFormatId = "multicoin_\(ethAddress)"
        try walletStore.addWallet(.mock(id: .multicoin(address: ethAddress), type: .multicoin, accounts: [.mock(chain: .ethereum, address: ethAddress)], order: 1))
        try db.insertLegacyWallet(id: "uuid-other", type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0xother")], order: 0)

        userDefaults.set(newFormatId, forKey: currentWalletKey)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let currentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(currentWalletId == newFormatId)
    }
}
