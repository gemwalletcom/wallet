// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

struct AddressRowView: View {
    private let model: AddressRowModel
    private let onCopy: VoidAction

    init(model: AddressRowModel, onCopy: VoidAction) {
        self.model = model
        self.onCopy = onCopy
    }

    var body: some View {
        ActionMenu(items: [.button(title: Localized.Common.copy, systemImage: SystemImage.copy, action: onCopy)]) {
            Text(model.address.preventingHyphenation)
                .textStyle(.body)
                .multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
                .frame(maxWidth: .infinity)
        }
    }
}
