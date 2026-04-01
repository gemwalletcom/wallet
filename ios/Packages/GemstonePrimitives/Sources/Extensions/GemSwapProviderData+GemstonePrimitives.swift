// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSwapProviderData
import Primitives

public extension Gemstone.GemSwapProviderData {
    func map() throws -> Primitives.SwapProviderData {
        try Primitives.SwapProviderData(
            provider: provider.map(),
            name: name,
            protocolName: protocolName,
        )
    }
}

extension Primitives.SwapProviderData {
    func map() throws -> Gemstone.GemSwapProviderData {
        try Gemstone.GemSwapProviderData(
            provider: provider.map(),
            name: name,
            protocolName: protocolName,
        )
    }
}
