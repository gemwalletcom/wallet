// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import Style
import SwiftUI

struct ValidatorSelectionView: View {
    private let value: ListItemValue<DelegationValidator>
    private let selection: String?
    private let action: ((DelegationValidator) -> Void)?

    init(
        value: ListItemValue<DelegationValidator>,
        selection: String?,
        action: ((DelegationValidator) -> Void)?,
    ) {
        self.value = value
        self.selection = selection
        self.action = action
    }

    var body: some View {
        Button {
            action?(value.value)
        } label: {
            HStack {
                ValidatorImageView(model: ValidatorViewModel(validator: value.value))
                    .assetBadge(value.value.id == selection ? Images.Wallets.selected : nil)
                ListItemView(
                    title: value.title,
                    subtitle: value.subtitle,
                )
            }
        }
        .contentShape(Rectangle())
    }
}
