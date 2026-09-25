// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemValidatorRow
import Style
import SwiftUI

struct ValidatorSelectionView: View {
    private let model: ValidatorViewModel
    private let isSelected: Bool
    private let action: () -> Void

    init(row: GemValidatorRow, isSelected: Bool, action: @escaping () -> Void) {
        model = ValidatorViewModel(row: row)
        self.isSelected = isSelected
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            HStack {
                ValidatorImageView(model: model)
                    .assetBadge(isSelected ? Images.Wallets.selected : nil)
                ListItemView(model: model.listItem)
            }
        }
        .contentShape(Rectangle())
    }
}
