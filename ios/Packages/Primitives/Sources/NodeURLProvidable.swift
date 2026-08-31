// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol NodeURLProvidable: Sendable {
    func node(for chain: Chain) -> URL
}
