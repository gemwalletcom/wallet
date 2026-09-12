// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct ValidatorImageView: View {
    private let model: ValidatorViewModel

    public init(model: ValidatorViewModel) {
        self.model = model
    }

    public var body: some View {
        if let providerImage = model.providerImage {
            providerImage
                .resizable()
                .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                .clipShape(Circle())
        } else {
            AsyncImageView(
                url: model.row.imageUrl.asURL,
                size: .image.asset,
                placeholder: .letter(model.row.placeholder.first ?? " "),
            )
        }
    }
}
