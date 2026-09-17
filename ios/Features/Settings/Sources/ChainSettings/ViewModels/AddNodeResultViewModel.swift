// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNodeCheck

struct AddNodeResultViewModel {
    private let result: GemNodeCheck

    init(result: GemNodeCheck) {
        self.result = result
    }

    var url: String {
        result.url
    }

    var fields: [ListItemField] {
        result.rows().map { ListItemField(title: $0.title, value: $0.text) }
    }
}
