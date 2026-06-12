// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct SupportAgentMessageGroup: View {
    let name: String
    let messages: [SupportMessageBubbleViewModel]

    var body: some View {
        VStack(alignment: .leading, spacing: .tiny) {
            Text(name)
                .font(.caption)
                .foregroundStyle(Colors.secondaryText)
            ForEach(messages) { message in
                HStack(spacing: .zero) {
                    SupportMessageBubble(model: message)
                    Spacer(minLength: .space32)
                }
            }
        }
    }
}
