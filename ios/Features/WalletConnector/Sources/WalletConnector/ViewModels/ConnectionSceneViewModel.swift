// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConnectionDetails
import Localization

public struct ConnectionSceneViewModel: Sendable {
    let details: GemConnectionDetails

    var title: String {
        Localized.WalletConnect.Connection.title
    }

    var disconnectTitle: String {
        Localized.WalletConnect.disconnect
    }
}
