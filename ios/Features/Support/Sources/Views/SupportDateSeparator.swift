// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesComponents
import Style
import SwiftUI

struct SupportDateSeparator: View {
    let date: Date

    var body: some View {
        Text(TransactionDateFormatter(date: date).section)
            .font(.caption)
            .foregroundStyle(Colors.secondaryText)
            .frame(maxWidth: .infinity)
            .padding(.vertical, .small)
    }
}
