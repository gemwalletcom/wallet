// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct ConfirmBalanceChangesView: View {
    let models: [ConfirmBalanceChangeViewModel]

    var body: some View {
        VStack(spacing: 0) {
            ForEach(Array(models.enumerated()), id: \.offset) { index, model in
                if index > 0 {
                    Divider()
                        .padding(.leading, .medium)
                }
                ListItemView(
                    title: TextValue(text: model.name, style: .body),
                    subtitle: model.amountTextValue,
                    imageStyle: .asset(assetImage: model.assetImage),
                )
                .padding(.horizontal, .medium)
                .padding(.vertical, .small)
            }
        }
        .background(Colors.white)
        .cornerRadius(.medium)
    }
}
