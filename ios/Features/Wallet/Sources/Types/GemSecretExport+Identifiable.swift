// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSecretExport

extension GemSecretExport: @retroactive Identifiable {
    public var id: String {
        switch self {
        case let .secret(kind): "secret.\(kind)"
        case let .privateKey(chain): "privateKey.\(chain)"
        case .privateKeyAccounts: "privateKeyAccounts"
        }
    }
}
