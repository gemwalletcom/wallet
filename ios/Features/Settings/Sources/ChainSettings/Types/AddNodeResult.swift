// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public struct AddNodeResult: Sendable {
    public let url: URL
    public let chainID: String
    public let blockNumber: BigInt
    public let latency: Latency
    public let isInSync: Bool

    public init(
        url: URL,
        chainID: String,
        blockNumber: BigInt,
        isInSync: Bool,
        latency: Latency,
    ) {
        self.url = url
        self.chainID = chainID
        self.blockNumber = blockNumber
        self.isInSync = isInSync
        self.latency = latency
    }
}
