// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct SupportDateSeparator: View {
    let title: String

    var body: some View {
        Text(title)
            .font(.caption)
            .foregroundStyle(Colors.secondaryText)
            .frame(maxWidth: .infinity)
            .padding(.vertical, .small)
    }
}
