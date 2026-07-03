// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct WalletDefiSection: View {
    var body: some View {
        Section {
            StateEmptyView(
                title: "Your DeFi positions will appear here",
                image: Images.EmptyContent.activity,
            )
            .padding(.vertical, .extraLarge)
        }
        .cleanListRow()
    }
}
