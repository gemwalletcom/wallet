// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct CollectionsPreviewView: View {
    private let content: CollectionsContent

    public init(content: CollectionsContent) {
        self.content = content
    }

    public var body: some View {
        ForEach(content.items) { item in
            NavigationLink(value: item.destination) {
                ListItemView(
                    title: item.model.title,
                    subtitle: item.model.count.map { String($0) },
                    imageStyle: ListItemImageStyle(
                        assetImage: item.model.assetImage,
                        imageSize: .image.asset,
                        cornerRadiusType: .custom(.small),
                    ),
                )
            }
        }

        if let unverifiedCount = content.unverifiedCount {
            NavigationLink(value: Scenes.UnverifiedCollections()) {
                ListItemView(
                    title: Localized.Asset.Verification.unverified,
                    subtitle: unverifiedCount,
                )
            }
        }
    }
}
