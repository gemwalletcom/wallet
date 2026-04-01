// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemWalletConnectionSessionAppMetadata {
    func map() -> WalletConnectionSessionAppMetadata {
        WalletConnectionSessionAppMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
        )
    }
}

public extension WalletConnectionSessionAppMetadata {
    func map() -> GemWalletConnectionSessionAppMetadata {
        GemWalletConnectionSessionAppMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
        )
    }

    var shortName: String {
        Gemstone.walletConnectAppShortName(metadata: map())
    }
}
