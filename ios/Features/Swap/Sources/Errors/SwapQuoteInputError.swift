// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SwapQuoteInputError: Error {
    case invalidAmount
    case formattingError
    case missingFromAsset
    case missingToAsset
}
