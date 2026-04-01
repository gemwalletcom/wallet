// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension AuthNonce {
    func map() -> GemAuthNonce {
        GemAuthNonce(nonce: nonce, timestamp: timestamp)
    }
}
