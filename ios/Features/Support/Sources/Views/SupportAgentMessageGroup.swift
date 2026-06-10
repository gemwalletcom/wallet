// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct SupportAgentMessageGroup: View {
    let name: String
    let messages: [SupportMessageBubbleViewModel]

    var body: some View {
        VStack(alignment: .leading, spacing: .tiny) {
            headerView
            ForEach(messages) { message in
                HStack(spacing: .zero) {
                    SupportMessageBubble(model: message)
                    Spacer(minLength: .space32)
                }
            }
        }
    }

    private var headerView: some View {
        HStack(spacing: .small) {
            AssetImageView(
                assetImage: AssetImage(
                    imageURL: nil,
                    placeholder: Images.Logo.logo,
                ),
                size: .image.small,
            )
            Text(name)
                .font(.caption)
                .foregroundStyle(Colors.secondaryText)
        }
    }
}
