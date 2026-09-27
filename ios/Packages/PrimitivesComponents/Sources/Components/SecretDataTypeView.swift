// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct SecretDataTypeView: View {
    private let type: SecretPhraseDataType

    init(type: SecretPhraseDataType) {
        self.type = type
    }

    var body: some View {
        switch type {
        case let .words(rows):
            SecretPhraseGridView(rows: rows)
        case let .privateKey(key):
            Text(key)
                .multilineTextAlignment(.center)
                .padding(.horizontal, .medium)
                .frame(maxWidth: .scene.content.maxWidth)
        }
    }
}
