// Copyright (c). Gem Wallet. All rights reserved.

import GRDB

extension ColumnDefinition {
    @discardableResult
    func referencesWallet() -> Self {
        references(WalletRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
    }

    @discardableResult
    func referencesAsset() -> Self {
        references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
    }
}
