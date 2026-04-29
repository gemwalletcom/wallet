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

                // We only handle `false` booleans to set our optional to `nil`
                // as we can't handle `true` for restoring the previous value.
                bindingOptional.wrappedValue = nil
            },
        )
    }
}

// TODO: - move Binding extension somewhere to SwiftUI extensions
public extension Binding {
    func mappedToBool<Wrapped: Sendable>() -> Binding<Bool> where Value == Wrapped? {
        Binding<Bool>(bindingOptional: self)
    }
}
