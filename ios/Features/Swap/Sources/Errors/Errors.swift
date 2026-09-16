// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSwapErrorDisplay

public enum SwapQuoteInputError: Error {
    case invalidAmount
    case formattingError
    case missingFromAsset
    case missingToAsset
}

extension GemSwapErrorDisplay: @retroactive Error {}
