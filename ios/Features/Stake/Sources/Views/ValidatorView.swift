// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import SwiftUI

public struct ValidatorView: View {
    private let model: ValidatorViewModel

    public init(model: ValidatorViewModel) {
        self.model = model
    }

    public var body: some View {
        HStack {
            ValidatorImageView(model: model)
            ListItemView(
                title: model.name,
                subtitle: model.aprModel.text,
            )
        }
    }
}
