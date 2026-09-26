// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Array where Element: Hashable {
    func asSet() -> Set<Element> {
        Set(self)
    }

    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

