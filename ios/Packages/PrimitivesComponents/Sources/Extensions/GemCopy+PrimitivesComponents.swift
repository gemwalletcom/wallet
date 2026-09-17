// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCopy
import GemstonePrimitives
import Primitives

public extension GemCopy {
    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(type: copyType, copyValue: value)
    }

    private var copyType: CopyType {
        switch kind {
        case let .address(chain): .address(Chain(core: chain).asset, address: display)
        case .secretPhrase: .secretPhrase
        case .privateKey: .privateKey
        }
    }
}
