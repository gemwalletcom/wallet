// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.linkTypeOrder
import Primitives

public extension LinkType {
    var order: Int {
        linkTypeOrder(linkType: rawValue).asInt
    }
}
