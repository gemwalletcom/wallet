// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemListSection
import Primitives
import PrimitivesComponents
import SwiftUI

struct WalletConnectReviewSection: View {
    let section: GemListSection

    var body: some View {
        Section {
            ForEach(Array(section.rows.enumerated()), id: \.offset) { _, row in
                if case .selectWallet = row.action() {
                    NavigationLink(value: Scenes.SelectWallet()) {
                        GemListRowView(row: row)
                    }
                } else {
                    GemListRowView(row: row)
                }
            }
        }
    }
}
