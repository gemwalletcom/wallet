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
                try addressName.record.insert(db, onConflict: .replace)
            }
        }
    }

    public func updateAddressNames(_ addressNames: [AddressName]) throws {
        if addressNames.isEmpty {
            return
        }
        try db.write { db in
            for addressName in addressNames {
                let reservedTypes = AddressType.allCases
                    .filter { $0.isLocal && $0 != addressName.type }
                    .map(\.rawValue)
                try AddressRecord
                    .filter(AddressRecord.Columns.chain == addressName.chain.rawValue)
                    .filter(AddressRecord.Columns.address == addressName.address)
                    .filter(!reservedTypes.contains(AddressRecord.Columns.type))
                    .updateAll(db, [
                        AddressRecord.Columns.name.set(to: addressName.name),
                        AddressRecord.Columns.type.set(to: addressName.type.rawValue),
                        AddressRecord.Columns.status.set(to: addressName.status.rawValue),
                        AddressRecord.Columns.imageUrl.set(to: addressName.imageUrl),
                    ])
                try addressName.record.insert(db, onConflict: .ignore)
            }
        }
    }

    public func deleteAddressNames(_ addressNames: [AddressName]) throws {
        try db.write { db in
            for addressName in addressNames {
                try AddressRecord
                    .filter(AddressRecord.Columns.chain == addressName.chain.rawValue)
                    .filter(AddressRecord.Columns.address == addressName.address)
                    .filter(AddressRecord.Columns.type == addressName.type.rawValue)
                    .deleteAll(db)
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
}
