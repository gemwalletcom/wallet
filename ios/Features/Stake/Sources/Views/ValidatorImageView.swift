// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemValidatorRow
import PrimitivesComponents
import Style
import SwiftUI

public struct ValidatorImageView: View {
    private let row: GemValidatorRow

    public init(row: GemValidatorRow) {
        self.row = row
    }

    public var body: some View {
        if let image = row.provider?.image {
            image
                .resizable()
                .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                .clipShape(Circle())
        } else {
            AsyncImageView(
                url: row.imageUrl.asURL,
                size: .image.asset,
                placeholder: .letter(row.placeholder.first ?? " "),
            )
        }
    }
}
