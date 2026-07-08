// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

struct AssetListItemView: View {
    let model: AssetListItemViewModel

    var body: some View {
        ListItemView(
            title: model.name,
            subtitle: model.count,
            imageStyle: .settings(assetImage: model.image),
        )
    }
}
