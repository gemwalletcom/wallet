// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecret
import Primitives
import Style
import SwiftUI

struct SecretDataTypeView: View {
    private let secret: GemWalletSecret

    init(secret: GemWalletSecret) {
        self.secret = secret
    }

    var body: some View {
        CaptureProtectedView {
            switch secret {
            case let .words(words):
                SecretPhraseGridView(words: words)
            case let .privateKey(key, _):
                Text(key.preventingHyphenation)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, .medium)
                    .frame(maxWidth: .scene.content.maxWidth)
            }
        }
    }
}
