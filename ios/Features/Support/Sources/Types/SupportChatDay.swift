// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSupportChatGroup

struct SupportChatDay: Identifiable {
    let date: Date
    let title: String
    let groups: [GemSupportChatGroup]

    var id: Date { date }
}
