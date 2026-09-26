// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSupportMessageRow
import Primitives
import Style
import SwiftUI

struct SupportAgentMessageGroup: View {
    let rows: [GemSupportMessageRow]
    let onRetry: (SupportMessage) -> Void
    let onImage: (SupportMessageImage) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: .tiny) {
            ForEach(rows, id: \.message.id) { row in
                HStack(spacing: .zero) {
                    SupportMessageBubble(row: row, onRetry: onRetry, onImage: onImage)
                    Spacer(minLength: .space32)
                }
            }
        }
    }
}
