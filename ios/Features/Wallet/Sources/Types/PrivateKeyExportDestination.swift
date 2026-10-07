// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecret

enum PrivateKeyExportDestination: Hashable {
    case accounts
    case secret(GemWalletSecret)
}
