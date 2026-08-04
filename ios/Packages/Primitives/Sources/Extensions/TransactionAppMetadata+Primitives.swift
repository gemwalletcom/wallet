// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension WalletConnectionSessionAppMetadata {
    var transactionAppMetadata: TransactionAppMetadata {
        TransactionAppMetadata(name: name, description: description, url: url, icon: icon)
    }
}

public extension TransactionAppMetadata {
    var iconURL: URL? {
        guard let icon, let iconURL = icon.asURL else { return .none }
        guard iconURL.host() == nil else { return iconURL }
        return url.flatMap { ($0 + icon).asURL }
    }
}
