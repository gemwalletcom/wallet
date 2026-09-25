// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@MainActor
@Observable
public final class InputValidationViewModel {
    public var text: String = "" {
        didSet {
            if text.isEmpty || text != oldValue {
                error = nil
            }
        }
    }

    public private(set) var error: (any Error)?

    public init() {}
}

// MARK: - Public

public extension InputValidationViewModel {
    func update(error: (any Error)?) {
        self.error = error
    }
}
