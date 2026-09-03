// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Wallet: Identifiable {}

public extension Wallet {
    var canSign: Bool {
        !isViewOnly
    }

    var isViewOnly: Bool {
        type == .view
    }

    var isMultiCoins: Bool {
        type == .multicoin
    }

    func account(for chain: Chain) throws -> Account {
        guard let account = accounts.filter({ $0.chain == chain }).first else {
            throw AnyError("account not found for chain: \(chain.rawValue)")
        }
        return account
    }
}
