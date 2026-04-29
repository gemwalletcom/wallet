// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Primitives

struct NFTAttributeViewModel: Identifiable {
    let id: String
    let name: String
    let value: String

    init(attribute: NFTAttribute, relativeDateFormatter: RelativeDateFormatter = RelativeDateFormatter()) {
        id = attribute.id
        name = attribute.name
        value = switch attribute.valueType ?? .string {
        case .timestamp:
            relativeDateFormatter.string(fromTimestampValue: attribute.value)
        case .string:
            attribute.value
        }
    }
}
