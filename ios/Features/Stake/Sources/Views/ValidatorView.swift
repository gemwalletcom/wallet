// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemValidatorRow
import SwiftUI

public struct ValidatorView: View {
    private let row: GemValidatorRow

    public init(row: GemValidatorRow) {
        self.row = row
    }

    public var body: some View {
        HStack {
            ValidatorImageView(row: row)
            ListItemView(model: row.listItem)
        }
    }
}
