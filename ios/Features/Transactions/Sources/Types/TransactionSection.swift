// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemTransactionDetailRow
import struct Gemstone.GemTransactionDetailSection
import Primitives
import PrimitivesComponents
import SwiftUI

extension GemTransactionDetailRow: @retroactive Identifiable {
    public var id: Self {
        self
    }
}

public extension ListSection where T == GemTransactionDetailRow {
    init(_ section: GemTransactionDetailSection) {
        self.init(id: "\(section.rows[0])", title: nil, image: nil, values: section.rows)
    }
}
