// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AddressStore: Sendable {
    
    let db: DatabaseQueue
    
    public init(db: DB) {
        self.db = db.dbQueue
    }
    
    public func addAddressNames(_ addressNames: [AddressName]) throws {
        try db.write { db in
            for addressName in addressNames {
                try AddressRecord(
                    chain: addressName.chain,
                    address: addressName.address,
                    name: addressName.name,
                    type: addressName.type,
                    status: addressName.status
                ).save(db, onConflict: .replace)
            }
        }
    }
    
    func deleteAddress(chain: Chain, address: String) throws -> Int {
        try db.write { db in
            try AddressRecord
                .filter(AddressRecord.Columns.chain == chain.rawValue)
                .filter(AddressRecord.Columns.address == address)
                .deleteAll(db)
        }
    }
    
    public func getAddressName(chain: Chain, address: String) throws -> AddressName? {
        try db.read { db in
            try AddressRecord
                .filter(AddressRecord.Columns.chain == chain.rawValue)
                .filter(AddressRecord.Columns.address == address)
                .fetchOne(db)?
                .mapToAddressName()
        }
    }

    public func addWalletAddresses(wallet: Wallet) throws {
        try db.write { db in
            for account in wallet.accounts {
                try AddressRecord(
                    chain: account.chain,
                    address: account.address,
                    walletId: wallet.id,
                    name: wallet.name,
                    type: .internalWallet,
                    status: .verified
                ).save(db, onConflict: .replace)
            }
        }
    }

    public func renameWalletAddresses(walletId: String, name: String) throws {
        let _ = try db.write { db in
            try AddressRecord
                .filter(AddressRecord.Columns.walletId == walletId)
                .updateAll(db, AddressRecord.Columns.name.set(to: name))
        }
    }
}
