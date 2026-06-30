// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct AssetListItemView: View {
    let model: AssetListItemViewModel

    var body: some View {
        ListItemView(
            title: model.name,
            titleStyle: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            subtitle: model.count,
            subtitleStyle: TextStyle(font: .callout, color: Colors.secondaryText, fontWeight: .semibold),
            imageStyle: .asset(assetImage: model.image),
        )
    }
}
