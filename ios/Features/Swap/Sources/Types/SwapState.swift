// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.SwapperQuote
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

public struct SwapState: Sendable {
    public var quotes: StateViewType<[SwapperQuote]>
    public var swapTransferData: StateViewType<GemTransferData>

    public init(
        quotes: StateViewType<[SwapperQuote]> = .noData,
        swapTransferData: StateViewType<GemTransferData> = .noData,
    ) {
        self.quotes = quotes
        self.swapTransferData = swapTransferData
    }

    public var isLoading: Bool {
        quotes.isLoading || swapTransferData.isLoading
    }

    public var error: (any Error)? {
        if case let .error(error) = swapTransferData {
            return error
        }
        if case let .error(error) = quotes {
            return error
        }
        return nil
    }
}
