// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct TransactionStateRequest: Sendable {
    public let id: String
    public let senderAddress: String
    public let block: Int
    public let createdAt: Date
    public let swapProvider: SwapProvider?

    public init(
        id: String,
        senderAddress: String,
        block: Int,
        createdAt: Date,
        swapProvider: SwapProvider? = nil,
    ) {
        self.id = id
        self.senderAddress = senderAddress
        self.block = block
        self.createdAt = createdAt
        self.swapProvider = swapProvider
    }
}
