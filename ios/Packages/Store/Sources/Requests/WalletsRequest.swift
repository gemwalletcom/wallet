// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.sortedWallets
import GemstonePrimitives
import GRDB
import Primitives

public struct WalletsRequest: DatabaseQueryable {
    private let isPinned: Bool

    public init(isPinned: Bool) {
        self.isPinned = isPinned
    }

    public func fetch(_ db: Database) throws -> [Wallet] {
        let wallets = try WalletRecord
            .including(all: WalletRecord.accounts)
            .filter(WalletRecord.Columns.isPinned == isPinned)
            .asRequest(of: WalletRecordInfo.self)
            .fetchAll(db)
            .map { $0.mapToWallet() }
        return try sortedWallets(wallets: wallets.map { try $0.json() }).map { try Wallet($0) }
    }
}

extension WalletsRequest: Equatable {}
