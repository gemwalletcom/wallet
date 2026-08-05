// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct SignMessage: Sendable, Equatable, Hashable {
    public let chain: String
    public let signType: SignDigestType
    public let data: Data

    public init(
        chain: String,
        signType: SignDigestType,
        data: Data,
    ) {
        self.chain = chain
        self.signType = signType
        self.data = data
    }
}
