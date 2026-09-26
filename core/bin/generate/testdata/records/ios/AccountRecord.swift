// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

struct AccountRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "wallets_accounts"

    enum Columns {
        static let walletId = Column("walletId")
    }

    var walletId: String
    var chain: Chain
    var address: String
    var extendedPublicKey: String?
    var index: Int = 0
    var derivationPath: String

    static let wallet = belongsTo(WalletRecord.self)
}
