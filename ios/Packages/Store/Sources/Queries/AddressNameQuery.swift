// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AddressNameQuery: DatabaseQueryable {
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
            .including(optional: AddressRecord.wallet)
            .asRequest(of: AddressNameInfo.self)
            .fetchOne(db)?
            .mapToAddressName()
    }
}

extension AddressNameQuery: Equatable {}
