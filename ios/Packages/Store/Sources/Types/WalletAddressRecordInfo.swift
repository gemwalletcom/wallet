// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct WalletAddressRecordInfo: FetchableRecord, Codable {
    var account: AccountRecord
    var wallet: WalletRecord
}

extension WalletAddressRecordInfo {
    func toWalletAddressItem() -> WalletAddressItem {
        WalletAddressItem(wallet: wallet.toWalletListItem(), address: account.address)
    }
}
