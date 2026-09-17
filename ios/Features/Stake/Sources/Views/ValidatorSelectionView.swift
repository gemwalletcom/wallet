// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import Style
import SwiftUI

struct ValidatorSelectionView: View {
    private let value: ListItemValue<DelegationValidator>
    private let validatorModel: ValidatorViewModel
    private let selection: String?
    private let action: ((DelegationValidator) -> Void)?

    init(
        value: ListItemValue<DelegationValidator>,
        validatorModel: ValidatorViewModel,
        selection: String?,
        action: ((DelegationValidator) -> Void)?,
    ) {
        self.value = value
        self.validatorModel = validatorModel
        self.selection = selection
        self.action = action
    }

    var body: some View {
        Button {
            action?(value.value)
        } label: {
            HStack {
                ValidatorImageView(model: validatorModel)
                    .assetBadge(value.value.id == selection ? Images.Wallets.selected : nil)
                ListItemView(model: value.listItem)
            }
        }
        .contentShape(Rectangle())
    }
}
