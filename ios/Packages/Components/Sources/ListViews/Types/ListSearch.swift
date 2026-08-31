// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct ListSearch<Item> {
    public let filter: (Item, String) -> Bool
    public let emptyContent: any EmptyContentViewable

    public init(
        filter: @escaping (Item, String) -> Bool,
        emptyContent: any EmptyContentViewable,
    ) {
        self.filter = filter
        self.emptyContent = emptyContent
    }
}
