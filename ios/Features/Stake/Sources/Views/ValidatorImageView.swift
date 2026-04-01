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
        switch model.validator.providerType {
        case .stake:
            AsyncImageView(
                url: model.imageUrl,
                size: .image.asset,
                placeholder: .letter(model.validator.name.first ?? " "),
            )
        case .earn:
            let image = model.image ?? Images.Logo.logo
            image
                .resizable()
                .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                .clipShape(Circle())
        }
    }
}
