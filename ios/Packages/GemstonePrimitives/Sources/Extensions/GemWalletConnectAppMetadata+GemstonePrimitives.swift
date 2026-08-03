// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemWalletConnectAppMetadata {
    func map() -> WalletConnectAppMetadata {
        WalletConnectAppMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
        )
    }
}

public extension WalletConnectAppMetadata {
    func map() -> GemWalletConnectAppMetadata {
        GemWalletConnectAppMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
        )
    }

    var shortName: String {
        Gemstone.walletConnectAppShortName(name: name)
    }
}
