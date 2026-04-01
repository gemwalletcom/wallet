// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Style
import SwiftUI

struct ConnectionView: View {
    @State private var isPresentingUrl: URL? = nil
    let model: WalletConnectionViewModel

    init(model: WalletConnectionViewModel) {
        self.model = model
    }

    var body: some View {
        HStack(spacing: .space12) {
            AsyncImageView(url: model.imageUrl, size: Sizing.image.app)
            VStack(alignment: .leading) {
                Text(model.nameText)
                    .font(.body)
                    .foregroundStyle(.primary)
                    .lineLimit(2)
                if let host = model.hostText {
                    Text(host)
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
        }
        .contextMenu {
            if let url = model.url {
                ContextMenuItem(
                    title: Localized.Settings.website,
                    systemImage: SystemImage.globe,
                ) {
                    isPresentingUrl = url
                }
            }
        }
        .safariSheet(url: $isPresentingUrl)
    }
}
