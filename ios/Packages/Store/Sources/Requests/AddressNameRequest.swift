// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AddressNameRequest: DatabaseQueryable {
    public let chain: Chain
    public let address: String

    public init(chain: Chain, address: String) {
        self.chain = chain
        self.address = address
    }

    public func fetch(_ db: Database) throws -> AddressName? {
        try AddressRecord
            .filter(AddressRecord.Columns.chain == chain.rawValue)
            .filter(AddressRecord.Columns.address == address)
            .fetchOne(db)?
            .mapToAddressName()
    }
}

extension AddressNameRequest: Equatable {}
