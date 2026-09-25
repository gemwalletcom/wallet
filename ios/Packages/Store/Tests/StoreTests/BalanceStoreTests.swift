// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct BalanceStoreTests {
    private let ethereum = AssetId(chain: .ethereum)
    private let solana = AssetId(chain: .solana)
    private let wallet = WalletId.multicoin(address: "0x1")
    private let otherWallet = WalletId.multicoin(address: "0x2")

    private func store() throws -> BalanceStore {
        let db = DB.mock(chains: [.ethereum, .solana])
        let walletStore = WalletStore(db: db)
        try walletStore.addWallet(.mock(id: wallet, accounts: [.mock(chain: .ethereum), .mock(chain: .solana)]))
        try walletStore.addWallet(.mock(id: otherWallet, accounts: [.mock(chain: .ethereum), .mock(chain: .solana)]))
        return BalanceStore(db: db)
    }

    @Test
    func addingABalanceTwiceKeepsTheStoredOne() throws {
        let store = try store()
        try store.addBalance(assetIds: [ethereum], isEnabled: true, for: wallet)
        try store.updateBalances([.mock(assetId: ethereum, available: 5)], for: wallet)

        try store.addBalance(assetIds: [ethereum], isEnabled: false, for: wallet)

        let record = try #require(try store.getBalanceRecord(walletId: wallet, assetId: ethereum))
        #expect(record.availableAmount == 5, "a second add never clears what the lane published")
        #expect(record.isEnabled, "a second add never flips the configuration")
    }

    @Test
    func updatingABalanceThatHasNoRowWritesNothing() throws {
        let store = try store()

        try store.updateBalances([.mock(assetId: ethereum, available: 5)], for: wallet)

        #expect(try store.getBalanceRecord(walletId: wallet, assetId: ethereum) == nil)
        #expect(try store.getBalances(walletId: wallet, assetIds: [ethereum]).isEmpty)
    }

    @Test
    func aWalletNeverSeesAnotherWalletsBalance() throws {
        let store = try store()
        try store.addBalance(assetIds: [ethereum], isEnabled: true, for: wallet)
        try store.addBalance(assetIds: [ethereum], isEnabled: true, for: otherWallet)

        try store.updateBalances([.mock(assetId: ethereum, available: 7)], for: wallet)
        _ = try store.setConfiguration(walletId: wallet, assetIds: [ethereum], configuration: .disabled)

        let other = try #require(try store.getBalanceRecord(walletId: otherWallet, assetId: ethereum))
        #expect(other.availableAmount == 0)
        #expect(other.isEnabled)
        #expect(try store.getEnabledAssetIds(walletId: otherWallet) == [ethereum])
        #expect(try store.getEnabledAssetIds(walletId: wallet).isEmpty)
    }

    @Test
    func aConfigurationThatChangesNothingWritesNothing() throws {
        let store = try store()
        try store.addBalance(assetIds: [ethereum], isEnabled: true, for: wallet)

        #expect(try store.setConfiguration(walletId: wallet, assetIds: [ethereum], configuration: AssetConfiguration(isEnabled: true, isPinned: false)) == 0)
        #expect(try store.setConfiguration(walletId: wallet, assetIds: [ethereum], configuration: AssetConfiguration(isEnabled: nil, isPinned: nil)) == 0)
        #expect(try store.setConfiguration(walletId: wallet, assetIds: [ethereum], configuration: .disabled) == 1)
        #expect(try store.setConfiguration(walletId: wallet, assetIds: [ethereum], configuration: .disabled) == 0)
    }

    @Test
    func onlyTheMissingBalancesAreAdded() throws {
        let store = try store()
        try store.addBalance(assetIds: [ethereum], isEnabled: true, for: wallet)

        try store.addMissingBalances(walletId: wallet, assetIds: [ethereum, solana])

        #expect(try store.getEnabledAssetIds(walletId: wallet) == [ethereum], "the asset that was already there keeps its configuration")
        #expect(try store.getBalances(walletId: wallet, assetIds: [ethereum, solana]).count == 2)
    }

    @Test
    func aBatchThatFailsPartWayLeavesNothingBehind() throws {
        let db = DB.mock(chains: [.ethereum])
        try WalletStore(db: db).addWallet(.mock(id: wallet, accounts: [.mock(chain: .ethereum)]))
        let store = BalanceStore(db: db)

        #expect(throws: Error.self) {
            try store.addBalance([.init(assetId: ethereum, isEnabled: true), .init(assetId: solana, isEnabled: true)], for: wallet)
        }

        #expect(try store.getBalances(walletId: wallet, assetIds: [ethereum, solana]).isEmpty, "the asset written before the failure is rolled back with it")
    }

    @Test
    @MainActor
    func aBatchOfUpdatesReachesAnObserverOnce() async throws {
        let store = try store()
        try store.addBalance(assetIds: [ethereum, solana], isEnabled: true, for: wallet)

        let observed = Observed()
        let walletId = wallet.id
        let observation = ValueObservation.tracking { db in
            try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId)
                .order(BalanceRecord.Columns.assetId)
                .fetchAll(db)
                .map { StoredBalance(assetId: $0.assetId, balance: $0.mapToBalance(), isActive: $0.isActive) }
        }

        await withCheckedContinuation { continuation in
            let cancellable = observation.start(in: store.db, scheduling: .immediate) { _ in } onChange: { values in
                if observed.append(values) == 2 {
                    continuation.resume()
                }
            }
            try? store.updateBalances(
                [.mock(assetId: ethereum, available: 1), .mock(assetId: solana, available: 2)],
                for: wallet,
            )
            observed.keep(cancellable)
        }

        #expect(observed.values.count == 2, "the first is the initial read and the whole batch follows as one change")
        #expect(observed.values.last?.map(\.balance.available) == [1, 2])
    }
}

private final class Observed: @unchecked Sendable {
    private(set) var values: [[StoredBalance]] = []
    private var cancellable: AnyDatabaseCancellable?

    @discardableResult
    func append(_ value: [StoredBalance]) -> Int {
        values.append(value)
        return values.count
    }

    func keep(_ cancellable: AnyDatabaseCancellable) {
        self.cancellable = cancellable
    }
}
