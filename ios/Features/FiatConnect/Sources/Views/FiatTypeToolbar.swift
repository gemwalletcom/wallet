// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import SwiftUI

struct FiatTypeToolbar: ToolbarContent {
    @Binding private var selectedType: FiatQuoteType

    init(selectedType: Binding<FiatQuoteType>) {
        _selectedType = selectedType
    }

    var body: some ToolbarContent {
        ToolbarItem(placement: .principal) {
            Picker("", selection: $selectedType) {
                Text(Localized.Wallet.buy)
                    .tag(FiatQuoteType.buy)
                Text(Localized.Wallet.sell)
                    .tag(FiatQuoteType.sell)
            }
            .pickerStyle(.segmented)
            .frame(width: 200)
        }
    }
}
