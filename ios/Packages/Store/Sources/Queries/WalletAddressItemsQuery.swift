// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct WalletAddressItemsQuery: DatabaseQueryable {
    private let chain: Chain

    public init(chain: Chain) {
        self.chain = chain
    }

    public func fetch(_ db: Database) throws -> [WalletAddressItem] {
        try AccountRecord
            .filter(AccountRecord.Columns.chain == chain.rawValue)
            .including(required: AccountRecord.wallet.forKey("wallet"))
            .asRequest(of: WalletAddressRecordInfo.self)
            .fetchAll(db)
            .map { $0.toWalletAddressItem() }
    }
}

extension WalletAddressItemsQuery: Equatable {}
