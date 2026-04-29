// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI

public extension View {
    @MainActor
    func presentationDetentsForCurrentDeviceSize(expandable: Bool = false) -> some View {
        switch DeviceSize.current {
        case .small:
            return presentationDetents([.large])
        case .medium, .large:
            if expandable {
                return presentationDetents([.medium, .large])
            }
            return presentationDetents([.medium])
        }
    }

    func enabled(_ value: Bool) -> some View {
        disabled(!value)
    }
}
