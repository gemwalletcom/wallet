// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import PrimitivesComponents
import Style
import SwiftUI

struct ConnectionView: View {
    @State private var isPresentingUrl: URL? = nil
    let model: ConnectionViewModel

    var body: some View {
        HStack(spacing: .space12) {
            AsyncImageView(url: model.iconUrl, size: Sizing.image.app)
            VStack(alignment: .leading) {
                Text(model.title)
                    .font(.body)
                    .foregroundStyle(.primary)
                    .lineLimit(2)
                if let host = model.host {
                    Text(host)
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
        }
        .contextMenu {
            if let url = model.websiteUrl {
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
