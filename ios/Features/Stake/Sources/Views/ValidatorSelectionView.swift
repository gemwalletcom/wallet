// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemValidatorRow
import Primitives
import Style
import SwiftUI

struct ValidatorSelectionView: View {
    private let value: ListItemValue<GemValidatorRow>
    private let validatorModel: ValidatorViewModel
    private let selection: String?
    private let action: ((GemValidatorRow) -> Void)?

    init(
        value: ListItemValue<GemValidatorRow>,
        validatorModel: ValidatorViewModel,
        selection: String?,
        action: ((GemValidatorRow) -> Void)?,
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
                    .assetBadge(value.value.validator.id == selection ? Images.Wallets.selected : nil)
                ListItemView(model: value.listItem)
            }
        }
        .contentShape(Rectangle())
    }
}
