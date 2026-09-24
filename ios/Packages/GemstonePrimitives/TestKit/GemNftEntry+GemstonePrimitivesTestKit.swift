// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemNftEntry
import enum Gemstone.GemNftItem
import struct Gemstone.GemNftRow

public extension GemNftEntry {
    static func mock(item: GemNftItem, id: String = UUID().uuidString) -> GemNftEntry {
        GemNftEntry(item: item, row: GemNftRow(id: id, title: "", imageUrl: "", countText: nil, isVerified: true))
    }
}
