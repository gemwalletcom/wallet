// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol SelectableSearchable: SelectableListAdoptable {
    var search: ListSearch<Item>? { get }
}
