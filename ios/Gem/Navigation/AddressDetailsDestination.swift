// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Primitives
import SwiftUI

struct AddressDetailsDestination: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    let chainAddress: ChainAddress

    var body: some View {
        AddressDetailsNavigationStack(model: viewModelFactory.addressDetailsScene(chainAddress: chainAddress))
    }
}
