// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSocialLink
import Primitives
import SwiftUI

public struct SocialLinksView: View {
    private let links: [GemSocialLink]

    public init(links: [GemSocialLink]) {
        self.links = links
    }

    public var body: some View {
        ForEach(links, id: \.url) { link in
            if let url = link.url.asURL {
                let view = ListItemView(model: link.listItem)
                if let deepLink = link.deepLink, UIApplication.shared.canOpenURL(deepLink) {
                    NavigationCustomLink(with: view) {
                        UIApplication.shared.open(deepLink)
                    }
                } else {
                    SafariNavigationLink(url: url) {
                        view
                    }
                }
            }
        }
    }
}
