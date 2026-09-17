// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

struct AddressCardView: View {
    private let model: AddressCardModel
    private let action: @MainActor @Sendable () -> Void

    init(model: AddressCardModel, action: @escaping @MainActor @Sendable () -> Void) {
        self.model = model
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(model.address.preventingHyphenation)
                .multilineTextAlignment(.center)
                .textStyle(TextStyle(font: .subheadline, color: Colors.secondaryText, fontWeight: .medium))
                .fixedSize(horizontal: false, vertical: true)
                .frame(maxWidth: .infinity)
                .padding(.medium)
                .background(
                    RoundedRectangle(cornerRadius: .medium)
                        .fill(Colors.listStyleColor),
                )
        }
        .buttonStyle(.scale)
    }
}
