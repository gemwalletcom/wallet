// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSlippageSuggestion
import GemstonePrimitives
import PrimitivesComponents

extension GemSlippageSuggestion: @retroactive Identifiable, SuggestionViewable {
    public var id: UInt32 {
        bps
    }

    public var title: String {
        percent.text()
    }

    public var inputValue: String {
        input
    }
}
