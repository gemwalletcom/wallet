// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemWalletConnectMessageRequest

extension GemWalletConnectMessageRequest: @retroactive Identifiable {
    public var id: String {
        session.id
    }
}
