// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct FullWidthSectionModifier: ViewModifier {
    public init() {}

    public func body(content: Content) -> some View {
        content
            .cleanListRow()
            .removeListSectionMargins()
    }
}

public extension View {
    func fullWidthSection() -> some View {
        modifier(FullWidthSectionModifier())
    }
}

private extension View {
    @ViewBuilder
    func removeListSectionMargins() -> some View {
        if #available(iOS 26.0, *) {
            listSectionMargins(.horizontal, .zero)
        } else {
            self
        }
    }
}
