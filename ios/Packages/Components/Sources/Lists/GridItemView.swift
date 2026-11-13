// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct GridItemView: View {
    private let title: TextValue
    private let value: TextValue

    public init(
        title: TextValue,
        value: TextValue,
    ) {
        self.title = title
        self.value = value
    }

    public init(field: ListItemField) {
        self.init(title: field.title, value: field.value)
    }

    public var body: some View {
        GridRow {
            Text(title.text)
                .textStyle(title.style)
            Text(value.text)
                .textStyle(value.style)
                .gridColumnAlignment(.trailing)
        }
    }
}
