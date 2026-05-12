// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum TransactionHeaderAction: Equatable, Sendable {
    case url(URL)
    case nft(assetId: String)
}
