// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Transfer

public struct TransferExecutorMock: TransferExecutable {
    public var error: Error?

    public init(error: Error? = nil) {
        self.error = error
    }

    public func execute(input _: TransferConfirmationInput) async throws {
        if let error {
            throw error
        }
    }
}
