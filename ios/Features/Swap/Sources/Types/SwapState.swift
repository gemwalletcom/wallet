// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.SwapperQuote
import Primitives

public struct SwapState: Sendable {
    public var quotes: StateViewType<[SwapperQuote]>
    public var swapTransferData: StateViewType<TransferData>

    public init(
        quotes: StateViewType<[SwapperQuote]> = .noData,
        swapTransferData: StateViewType<TransferData> = .noData,
    ) {
        self.quotes = quotes
        self.swapTransferData = swapTransferData
    }

    public var isLoading: Bool {
        quotes.isLoading || swapTransferData.isLoading
    }

    public var error: (any Error)? {
        if case let .error(error) = quotes {
            return error
        }
        if case let .error(error) = swapTransferData {
            return error
        }
        return nil
    }
}
