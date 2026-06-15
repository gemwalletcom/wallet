// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct SupportUserMessageGroup: View {
    let messages: [SupportMessageBubbleViewModel]

    var body: some View {
        VStack(alignment: .trailing, spacing: .tiny) {
            ForEach(messages) { message in
                HStack(alignment: .center, spacing: .small) {
                    Spacer(minLength: .space32)
                    if message.isFailed {
                        Image(systemName: SystemImage.errorOccurred)
                            .font(.body)
                            .foregroundStyle(Colors.red)
                    }
                    SupportMessageBubble(model: message)
                }
            }
        }
    }
}
