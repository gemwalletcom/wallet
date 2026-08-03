// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PaymentDataCollectionRequest: Identifiable, Sendable {
    public let id: String
    public let url: URL

    public init(id: String, url: URL) {
        self.id = id
        self.url = url
    }
}
