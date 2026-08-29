// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletConnectRulesService
import Primitives

private let walletConnectRules = GemWalletConnectRulesService()

public extension Primitives.ApplicationMetadata {
    var shortName: String {
        guard let metadata = try? json() else { return name }
        return walletConnectRules.metadataShortName(metadata: metadata)
    }
}
