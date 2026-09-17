// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Style
import SwiftUI

public extension View {
    func sheetPresentation(
        _ detents: Set<PresentationDetent>,
        dragIndicator: Visibility = .automatic,
    ) -> some View {
        presentationDetents(detents)
            .presentationDragIndicator(dragIndicator)
            .presentationBackground(Colors.grayBackground)
    }

    func enabled(_ value: Bool) -> some View {
        disabled(!value)
    }
}

public extension Set<PresentationDetent> {
    @MainActor
    static func forCurrentDeviceSize(expandable: Bool = false) -> Set<PresentationDetent> {
        switch DeviceSize.current {
        case .small: [.large]
        case .medium, .large: expandable ? [.medium, .large] : [.medium]
        }
    }
}
