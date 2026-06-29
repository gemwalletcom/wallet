// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct ContactStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func addContact(_ contact: Contact, addresses: [ContactAddress]) throws {
        try db.write { db in
            try contact.record.insert(db)
            for address in addresses {
                try address.record.insert(db)
            }
        }
    }

    public func updateContact(_ contact: Contact, deleteAddressIds: [String], addresses: [ContactAddress]) throws {
        try db.write { db in
            try ContactRecord
                .filter(ContactRecord.Columns.id == contact.id)
                .updateAll(db, [
                    ContactRecord.Columns.name.set(to: contact.name),
                    ContactRecord.Columns.description.set(to: contact.description),
                    ContactRecord.Columns.updatedAt.set(to: contact.updatedAt),
                ])

            if deleteAddressIds.isNotEmpty {
                let removed = try ContactAddressRecord
                    .filter(deleteAddressIds.contains(ContactAddressRecord.Columns.id))
                    .fetchAll(db)
                try deleteAddressNames(db, for: removed)
                try ContactAddressRecord
                    .filter(deleteAddressIds.contains(ContactAddressRecord.Columns.id))
                    .deleteAll(db)
            }

            for address in addresses {
                try address.record.upsert(db)
            }
        }
    }

    public func getAddressIds(contactId: String) throws -> [String] {
        try db.read { db in
            try ContactAddressRecord
                .filter(ContactAddressRecord.Columns.contactId == contactId)
                .fetchAll(db)
                .map(\.id)
        }
    }

    @discardableResult
    public func deleteContact(id: String) throws -> Bool {
        try db.write { db in
            let records = try ContactAddressRecord
                .filter(ContactAddressRecord.Columns.contactId == id)
                .fetchAll(db)
            try deleteAddressNames(db, for: records)

            return try ContactRecord.deleteOne(db, key: id)
        }
    }
}

// MARK: - Private

extension ContactStore {
    private func deleteAddressNames(_ db: Database, for records: [ContactAddressRecord]) throws {
        let addressesByChain = records.reduce(into: [Chain: [String]]()) { $0[$1.chain, default: []].append($1.address) }

        for (chain, addresses) in addressesByChain {
            try AddressRecord
                .filter(AddressRecord.Columns.chain == chain.rawValue)
                .filter(addresses.contains(AddressRecord.Columns.address))
                .filter(AddressRecord.Columns.type == AddressType.contact.rawValue)
                .deleteAll(db)
        }
    }
}
