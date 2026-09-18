// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCopy

public extension GemCopy {
    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(content: self)
    }
}
