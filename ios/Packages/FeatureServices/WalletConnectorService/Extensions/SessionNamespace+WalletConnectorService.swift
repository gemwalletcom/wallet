// Copyright (c). Gem Wallet. All rights reserved.

import ReownWalletKit

extension [String: SessionNamespace] {
    func adding(_ account: Account) -> Self {
        guard var namespace = self[account.namespace] else {
            return self
        }
        namespace.chains = namespace.chains.map { $0 + [account.blockchain] }
        namespace.accounts.append(account)
        var namespaces = self
        namespaces[account.namespace] = namespace
        return namespaces
    }
}
