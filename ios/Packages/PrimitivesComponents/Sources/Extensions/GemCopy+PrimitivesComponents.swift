// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCopy
import Primitives

public extension GemCopy {
    var copyValue: CopyValue {
        switch kind {
        case let .address(chain): .address(value: value, chain: Chain(core: chain))
        case .plain, .secretPhrase, .privateKey: .plain(value)
        }
    }
}
