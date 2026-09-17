// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct SelectFilterView: View {
    private let typeModel: any FilterTypeRepresentable
    private let action: @MainActor @Sendable () -> Void

    public init(
        typeModel: any FilterTypeRepresentable,
        action: @escaping @MainActor @Sendable () -> Void,
    ) {
        self.typeModel = typeModel
        self.action = action
    }

    public var body: some View {
        NavigationCustomLink(
            with: ListItemView(model: typeModel.listItem),
            action: action,
        )
    }
}

#Preview {
    SelectFilterView(
        typeModel: ChainsFilterTypeViewModel(type: .allChains),
        action: {},
    )
}
