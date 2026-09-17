// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct NameRecordView: View {
    let model: NameRecordViewModel

    public init(model: NameRecordViewModel) {
        self.model = model
    }

    public var body: some View {
        VStack(alignment: .center, spacing: .zero) {
            if model.isResolving {
                LoadingView()
            } else if let image = model.resolveImage {
                image
            }
        }.frame(width: .space16, height: .space16)
    }
}
