// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemConnection
import Localization
import PrimitivesComponents
import Style
import SwiftUI

struct ConnectionView: View {
    @State private var isPresentingUrl: URL? = nil
    let connection: GemConnection

    var body: some View {
        HStack(spacing: .space12) {
            AsyncImageView(url: connection.row.iconUrl.flatMap(URL.init(string:)), size: Sizing.image.app)
            VStack(alignment: .leading) {
                Text(connection.row.title)
                    .font(.body)
                    .foregroundStyle(.primary)
                    .lineLimit(2)
                if let host = connection.row.host {
                    Text(host)
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
        }
        .contextMenu {
            if let url = URL(string: connection.connection.session.metadata.url) {
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
