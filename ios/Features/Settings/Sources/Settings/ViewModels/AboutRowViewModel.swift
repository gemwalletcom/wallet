// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Components

struct AboutRowViewModel: Identifiable {
    enum Kind {
        case link(URL)
        case community
        case version
    }

    let id: String
    let kind: Kind
    let model: ListItemModel
}
