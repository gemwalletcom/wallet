// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemGroup
import Primitives
import PrimitivesComponents
import SwiftUI

struct WalletConnectReviewSections: View {
    let groups: [GemGroup]

    var body: some View {
        GemGroupSections(rows: groups) { row in
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
