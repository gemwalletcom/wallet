// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct SupportUserMessageGroup: View {
    let messages: [SupportMessageBubbleViewModel]

    var body: some View {
        VStack(alignment: .trailing, spacing: .tiny) {
            ForEach(messages) { message in
                HStack(spacing: .zero) {
                    Spacer(minLength: .space32)
                    SupportMessageBubble(model: message)
                }
            }
        }
    }
}
