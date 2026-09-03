// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Chain {
    var defaultBaseUrl: URL {
        NodeURL.url(chain: self, region: .us)
    }
}
