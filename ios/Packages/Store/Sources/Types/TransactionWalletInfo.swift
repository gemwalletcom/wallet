// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct WalletTransactionInfo: FetchableRecord, Decodable {
    var transaction: TransactionRecord
    var wallet: WalletRecord
    var accounts: [AccountRecord]

    var transactionWallet: TransactionWallet {
        TransactionWallet(
            transaction: transaction.mapToTransaction(),
            wallet: WalletRecordInfo(wallet: wallet, accounts: accounts).mapToWallet(),
        )
    }
}
