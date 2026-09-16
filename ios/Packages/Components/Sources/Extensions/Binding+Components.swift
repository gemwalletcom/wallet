// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI

public extension Binding where Value == Bool {
    init(bindingOptional: Binding<(some Sendable)?>) {
        self.init(
            get: {
                bindingOptional.wrappedValue != nil
            },
            set: { newValue in
                guard newValue == false else { return }
                bindingOptional.wrappedValue = nil
            },
        )
    }
}

public extension Binding {
    func mappedToBool<Wrapped: Sendable>() -> Binding<Bool> where Value == Wrapped? {
        Binding<Bool>(bindingOptional: self)
    }
}
