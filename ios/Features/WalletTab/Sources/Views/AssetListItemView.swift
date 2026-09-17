// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

struct AssetListItemView: View {
    let model: AssetListItemViewModel

    var body: some View {
        ListItemView(model: model.listItem)
    }
}
