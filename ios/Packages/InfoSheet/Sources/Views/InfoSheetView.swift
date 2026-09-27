// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct InfoSheetView: View {
    private let model: InfoSheetModel

    init(model: InfoSheetModel) {
        self.model = model
    }

    var body: some View {
        VStack(spacing: .medium) {
            Group {
                switch model.image {
                case let .image(image):
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                case let .assetImage(asset):
                    AssetImageView(
                        assetImage: asset,
                        size: .image.semiExtraLarge,
                    )
                case nil: EmptyView()
                }
            }
            .frame(size: .image.large)

            VStack(spacing: .small) {
                Text(model.title)
                    .textStyle(model.titleStyle)
                Text(.init(model.description))
                    .textStyle(model.descriptionStyle)
            }
            .multilineTextAlignment(.center)
            .minimumScaleFactor(0.85)
        }
        .frame(maxWidth: .infinity, alignment: .top)
    }
}
